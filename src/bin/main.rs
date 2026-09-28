#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration as EmbassyDuration, Instant as EmbassyInstant, Timer};
use embedded_io_async::Write;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use esp_hal_dhcp_server::{
    Ipv4Addr, run_dhcp_server,
    structs::{DhcpLease, DhcpLeaser, DhcpServerConfig},
};
use esp_println::println;
use esp_radio::wifi::{
    Config as WifiConfig, ControllerConfig, Interface, WifiController, ap::AccessPointConfig,
};
use rc_car::main::controller::{RCCarConfig, RCCarController};
use static_cell::StaticCell;

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ════════════════════════════════════════════════════════════════
// Biến chia sẻ giữa Web Server và Vòng lặp điều khiển xe (Atomics)
// ════════════════════════════════════════════════════════════════

static CAR_STEER: AtomicI32 = AtomicI32::new(0); // -100 đến 100
static CAR_THROTTLE: AtomicI32 = AtomicI32::new(0); // -100 đến 100
static CAR_LAST_CMD_MS: AtomicU32 = AtomicU32::new(0); // Timestamp của lệnh cuối
static CAR_ACTIVE_CONTROL: AtomicBool = AtomicBool::new(false); // Đang nhận lệnh điều khiển
static CAR_WIFI_ACTIVE: AtomicBool = AtomicBool::new(false); // Trạng thái sóng Wi-Fi
static CAR_EMERGENCY_STOP: AtomicBool = AtomicBool::new(false); // Dừng khẩn cấp
static CAR_SPEED_LEFT: AtomicI32 = AtomicI32::new(0); // Tốc độ động cơ trái
static CAR_SPEED_RIGHT: AtomicI32 = AtomicI32::new(0); // Tốc độ động cơ phải
static CAR_UPTIME_SECS: AtomicU32 = AtomicU32::new(0); // Thời gian hoạt động (giây)

// ════════════════════════════════════════════════════════════════
// DHCP Leaser
// ════════════════════════════════════════════════════════════════

struct SmartDhcpLeaser {
    client_ip: Ipv4Addr,
}

impl SmartDhcpLeaser {
    fn new(client_ip: Ipv4Addr) -> Self {
        Self { client_ip }
    }
}

impl DhcpLeaser for SmartDhcpLeaser {
    fn get_lease(&mut self, mac: [u8; 16]) -> Option<DhcpLease> {
        Some(DhcpLease {
            ip: self.client_ip,
            mac,
            expires: EmbassyInstant::now() + EmbassyDuration::from_secs(3600),
        })
    }

    fn next_lease(&mut self) -> Option<Ipv4Addr> {
        Some(self.client_ip)
    }

    fn add_lease(&mut self, ip: Ipv4Addr, _mac: [u8; 16], _expires: EmbassyInstant) -> bool {
        ip == self.client_ip
    }

    fn remove_lease(&mut self, _mac: [u8; 16]) -> bool {
        true
    }
}

// ════════════════════════════════════════════════════════════════
// Embassy Tasks: Network & Servers
// ════════════════════════════════════════════════════════════════

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn dhcp_task(stack: Stack<'static>) {
    let mut leaser = SmartDhcpLeaser::new(Ipv4Addr::new(192, 168, 4, 2));
    let config = DhcpServerConfig {
        ip: Ipv4Addr::new(192, 168, 4, 1),
        lease_time: EmbassyDuration::from_secs(3600),
        gateways: &[Ipv4Addr::new(192, 168, 4, 1)],
        subnet: Some(Ipv4Addr::new(255, 255, 255, 0)),
        dns: &[Ipv4Addr::new(192, 168, 4, 1)],
        use_captive_portal: true,
    };
    let _ = run_dhcp_server(stack, config, &mut leaser).await;
}

#[embassy_executor::task]
async fn dns_task(stack: Stack<'static>) {
    use embassy_net::udp::{PacketMetadata, UdpSocket};
    use embassy_net::{IpAddress, IpEndpoint};

    let mut rx_buffer = [0u8; 512];
    let mut tx_buffer = [0u8; 512];
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_meta = [PacketMetadata::EMPTY; 4];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    if socket
        .bind(IpEndpoint::new(IpAddress::v4(0, 0, 0, 0), 53))
        .is_err()
    {
        return;
    }

    let mut buf = [0u8; 512];
    let mut resp = [0u8; 512];

    loop {
        if let Ok((n, meta)) = socket.recv_from(&mut buf).await {
            if n >= 12 {
                resp[..n].copy_from_slice(&buf[..n]);
                resp[2] = 0x81;
                resp[3] = 0x80;
                resp[6] = 0x00;
                resp[7] = 0x01;
                resp[8] = 0x00;
                resp[9] = 0x00;
                resp[10] = 0x00;
                resp[11] = 0x00;

                let idx = n;
                if idx + 16 <= resp.len() {
                    resp[idx] = 0xc0;
                    resp[idx + 1] = 0x0c;
                    resp[idx + 2] = 0x00;
                    resp[idx + 3] = 0x01;
                    resp[idx + 4] = 0x00;
                    resp[idx + 5] = 0x01;
                    resp[idx + 6] = 0x00;
                    resp[idx + 7] = 0x00;
                    resp[idx + 8] = 0x00;
                    resp[idx + 9] = 0x3c;
                    resp[idx + 10] = 0x00;
                    resp[idx + 11] = 0x04;
                    resp[idx + 12] = 192;
                    resp[idx + 13] = 168;
                    resp[idx + 14] = 4;
                    resp[idx + 15] = 1;
                    let _ = socket.send_to(&resp[..idx + 16], meta.endpoint).await;
                }
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Web Server Task: Controller Dashboard & API
// ════════════════════════════════════════════════════════════════

fn parse_query_param(request: &str, key: &str) -> Option<i32> {
    if let Some(pos) = request.find(key) {
        let after = &request[pos + key.len()..];
        let end = after
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(after.len());
        after[..end].parse::<i32>().ok()
    } else {
        None
    }
}

#[embassy_executor::task]
async fn web_server_task(stack: Stack<'static>) {
    let mut rx_buffer = [0u8; 2048];
    let mut tx_buffer = [0u8; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(EmbassyDuration::from_secs(5)));

        if socket.accept(80).await.is_err() {
            Timer::after(EmbassyDuration::from_millis(50)).await;
            continue;
        }

        let mut buf = [0u8; 1024];
        let n = match socket.read(&mut buf).await {
            Ok(n) if n > 0 => n,
            _ => continue,
        };

        let request = core::str::from_utf8(&buf[..n]).unwrap_or("");
        let now_sec = CAR_UPTIME_SECS.load(Ordering::Relaxed);
        let now_ms = now_sec.wrapping_mul(1000);

        if request.contains("GET /api/control") {
            let steer = parse_query_param(request, "s=")
                .unwrap_or(0)
                .clamp(-100, 100);
            let throttle = parse_query_param(request, "t=")
                .unwrap_or(0)
                .clamp(-100, 100);

            CAR_STEER.store(steer, Ordering::Relaxed);
            CAR_THROTTLE.store(throttle, Ordering::Relaxed);
            CAR_LAST_CMD_MS.store(now_ms, Ordering::Relaxed);

            let is_moving = steer != 0 || throttle != 0;
            CAR_ACTIVE_CONTROL.store(is_moving, Ordering::Relaxed);

            let left = CAR_SPEED_LEFT.load(Ordering::Relaxed);
            let right = CAR_SPEED_RIGHT.load(Ordering::Relaxed);

            let json_body = format!(
                "{{\"ok\":true,\"steer\":{},\"throttle\":{},\"left\":{},\"right\":{}}}",
                steer, throttle, left, right
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                json_body.len(),
                json_body
            );
            let _ = socket.write_all(response.as_bytes()).await;
        } else if request.contains("GET /api/stop") {
            CAR_STEER.store(0, Ordering::Relaxed);
            CAR_THROTTLE.store(0, Ordering::Relaxed);
            CAR_ACTIVE_CONTROL.store(false, Ordering::Relaxed);

            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"ok\":true,\"stopped\":true}";
            let _ = socket.write_all(response.as_bytes()).await;
        } else if request.contains("GET /api/estop") {
            let cur = CAR_EMERGENCY_STOP.load(Ordering::Relaxed);
            CAR_EMERGENCY_STOP.store(!cur, Ordering::Relaxed);
            if !cur {
                CAR_STEER.store(0, Ordering::Relaxed);
                CAR_THROTTLE.store(0, Ordering::Relaxed);
                CAR_ACTIVE_CONTROL.store(false, Ordering::Relaxed);
            }

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{{\"ok\":true,\"estop\":{}}}",
                !cur
            );
            let _ = socket.write_all(response.as_bytes()).await;
        } else if request.contains("GET /api/status") {
            let left = CAR_SPEED_LEFT.load(Ordering::Relaxed);
            let right = CAR_SPEED_RIGHT.load(Ordering::Relaxed);
            let active = CAR_ACTIVE_CONTROL.load(Ordering::Relaxed);
            let estop = CAR_EMERGENCY_STOP.load(Ordering::Relaxed);

            let json_body = format!(
                "{{\"left\":{},\"right\":{},\"active\":{},\"estop\":{},\"uptime\":{}}}",
                left, right, active, estop, now_sec
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                json_body.len(),
                json_body
            );
            let _ = socket.write_all(response.as_bytes()).await;
        } else {
            // HTML Controller Interface
            let html = r#"<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>ESP32 RC Car Controller</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; -webkit-tap-highlight-color: transparent; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0b0f19;
            color: #f1f5f9;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 12px;
            user-select: none;
            touch-action: manipulation;
        }
        .header {
            width: 100%;
            max-width: 420px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px 14px;
            background: #1e293b;
            border-radius: 12px;
            margin-bottom: 12px;
            border: 1px solid #334155;
        }
        .title { font-size: 1.1rem; font-weight: 700; color: #38bdf8; display: flex; align-items: center; gap: 6px; }
        .pill {
            padding: 4px 10px;
            font-size: 0.75rem;
            font-weight: 600;
            border-radius: 20px;
            background: #065f46;
            color: #34d399;
            display: flex;
            align-items: center;
            gap: 5px;
        }
        .dot { width: 8px; height: 8px; border-radius: 50%; background: #34d399; animation: pulse 1.5s infinite; }
        @keyframes pulse { 0% { opacity: 0.4; } 50% { opacity: 1; } 100% { opacity: 0.4; } }

        /* Telemetry Cards */
        .hud {
            width: 100%;
            max-width: 420px;
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 8px;
            margin-bottom: 14px;
        }
        .card {
            background: #1e293b;
            border: 1px solid #334155;
            padding: 10px;
            border-radius: 10px;
            text-align: center;
        }
        .card-label { font-size: 0.72rem; color: #94a3b8; text-transform: uppercase; margin-bottom: 4px; }
        .card-val { font-size: 1.25rem; font-weight: 800; font-family: monospace; color: #38bdf8; }

        /* Speed Preset Selectors */
        .speed-panel {
            width: 100%;
            max-width: 420px;
            display: flex;
            gap: 8px;
            margin-bottom: 16px;
        }
        .speed-btn {
            flex: 1;
            padding: 8px;
            background: #1e293b;
            border: 1px solid #475569;
            color: #94a3b8;
            border-radius: 8px;
            font-weight: 700;
            font-size: 0.85rem;
            cursor: pointer;
            transition: all 0.2s;
        }
        .speed-btn.active {
            background: #0284c7;
            color: white;
            border-color: #38bdf8;
            box-shadow: 0 0 10px rgba(56, 189, 248, 0.4);
        }

        /* D-Pad Controller */
        .controller {
            width: 100%;
            max-width: 320px;
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            grid-template-rows: repeat(3, 90px);
            gap: 12px;
            margin: auto 0;
        }
        .ctrl-btn {
            background: linear-gradient(145deg, #1e293b, #0f172a);
            border: 2px solid #334155;
            border-radius: 18px;
            color: #f8fafc;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            font-size: 1.7rem;
            cursor: pointer;
            box-shadow: 0 4px 12px rgba(0,0,0,0.5);
            transition: all 0.08s ease;
        }
        .ctrl-btn span { font-size: 0.72rem; font-weight: 700; margin-top: 2px; text-transform: uppercase; color: #94a3b8; }
        .ctrl-btn:active, .ctrl-btn.pressed {
            transform: scale(0.93);
            background: #0284c7;
            border-color: #38bdf8;
            box-shadow: 0 0 16px rgba(56, 189, 248, 0.6);
            color: white;
        }
        .ctrl-btn:active span, .ctrl-btn.pressed span { color: white; }
        .btn-stop {
            background: linear-gradient(145deg, #334155, #1e293b);
            border-color: #64748b;
            color: #f87171;
        }
        .btn-stop:active, .btn-stop.pressed {
            background: #dc2626;
            border-color: #ef4444;
            color: white;
            box-shadow: 0 0 16px rgba(239, 68, 68, 0.6);
        }

        /* E-Stop Bar */
        .bottom-bar {
            width: 100%;
            max-width: 420px;
            margin-top: 18px;
        }
        .estop-btn {
            width: 100%;
            padding: 12px;
            background: #7f1d1d;
            border: 1px solid #b91c1c;
            border-radius: 10px;
            color: #fecaca;
            font-size: 0.95rem;
            font-weight: 800;
            text-transform: uppercase;
            letter-spacing: 1px;
            cursor: pointer;
            transition: all 0.2s;
        }
        .estop-btn.active {
            background: #dc2626;
            color: white;
            box-shadow: 0 0 20px rgba(220, 38, 38, 0.8);
            animation: pulse-red 1s infinite;
        }
        @keyframes pulse-red { 0% { opacity: 0.7; } 50% { opacity: 1; } 100% { opacity: 0.7; } }
        .hint {
            font-size: 0.75rem;
            color: #64748b;
            text-align: center;
            margin-top: 8px;
        }
    </style>
</head>
<body>
    <div class="header">
        <div class="title">🏎️ ESP32 RC CAR</div>
        <div class="pill"><div class="dot"></div> <span id="status-pill">KẾT NỐI SẴN SÀNG</span></div>
    </div>

    <div class="hud">
        <div class="card">
            <div class="card-label">Motor Trái (L)</div>
            <div class="card-val" id="val-left">0%</div>
        </div>
        <div class="card">
            <div class="card-label">Motor Phải (R)</div>
            <div class="card-val" id="val-right">0%</div>
        </div>
    </div>

    <div class="speed-panel">
        <button class="speed-btn" onclick="setSpeed(35, this)">35% Chậm</button>
        <button class="speed-btn active" onclick="setSpeed(70, this)">70% Chuẩn</button>
        <button class="speed-btn" onclick="setSpeed(100, this)">100% Turbo</button>
    </div>

    <div class="controller">
        <div></div>
        <div class="ctrl-btn" id="btn-fwd" data-s="0" data-t="1">⬆️<span>Tiến</span></div>
        <div></div>

        <div class="ctrl-btn" id="btn-left" data-s="-1" data-t="0.75">⬅️<span>Trái</span></div>
        <div class="ctrl-btn btn-stop" id="btn-stop" data-s="0" data-t="0">⏹️<span>Dừng</span></div>
        <div class="ctrl-btn" id="btn-right" data-s="1" data-t="0.75">➡️<span>Phải</span></div>

        <div></div>
        <div class="ctrl-btn" id="btn-bwd" data-s="0" data-t="-1">⬇️<span>Lùi</span></div>
        <div></div>
    </div>

    <div class="bottom-bar">
        <button class="estop-btn" id="btn-estop" onclick="toggleEstop()">🛑 Phanh Khẩn Cấp (E-Stop)</button>
        <div class="hint">Đè nút trên màn hình hoặc dùng phím mũi tên bàn phím.</div>
    </div>

    <script>
        let currentPower = 70;
        let activeInterval = null;
        let curS = 0, curT = 0;

        function setSpeed(val, el) {
            currentPower = val;
            document.querySelectorAll('.speed-btn').forEach(b => b.classList.remove('active'));
            el.classList.add('active');
        }

        async function sendCommand(s, t) {
            try {
                const res = await fetch(`/api/control?s=${s}&t=${t}`);
                if (res.ok) {
                    const data = await res.json();
                    document.getElementById('val-left').innerText = data.left + '%';
                    document.getElementById('val-right').innerText = data.right + '%';
                    const pill = document.getElementById('status-pill');
                    if (s !== 0 || t !== 0) {
                        pill.innerText = 'ĐANG ĐIỀU KHIỂN';
                        pill.style.color = '#38bdf8';
                    } else {
                        pill.innerText = 'ĐANG CHỜ LỆNH';
                        pill.style.color = '#34d399';
                    }
                }
            } catch (e) {}
        }

        async function sendStop() {
            if (activeInterval) {
                clearInterval(activeInterval);
                activeInterval = null;
            }
            curS = 0; curT = 0;
            document.querySelectorAll('.ctrl-btn').forEach(b => b.classList.remove('pressed'));
            try {
                await fetch('/api/stop');
                document.getElementById('val-left').innerText = '0%';
                document.getElementById('val-right').innerText = '0%';
                document.getElementById('status-pill').innerText = 'ĐÃ DỪNG';
            } catch (e) {}
        }

        async function toggleEstop() {
            try {
                const res = await fetch('/api/estop');
                const data = await res.json();
                const btn = document.getElementById('btn-estop');
                if (data.estop) {
                    btn.classList.add('active');
                    btn.innerText = '⚠️ ĐANG KHÓA KHẨN CẤP (BẤM MỞ)';
                    document.getElementById('status-pill').innerText = 'E-STOP BẬT';
                } else {
                    btn.classList.remove('active');
                    btn.innerText = '🛑 Phanh Khẩn Cấp (E-Stop)';
                    document.getElementById('status-pill').innerText = 'ĐÃ MỞ KHÓA';
                }
            } catch (e) {}
        }

        function startDrive(btn) {
            const rawS = parseFloat(btn.dataset.s || 0);
            const rawT = parseFloat(btn.dataset.t || 0);

            if (rawS === 0 && rawT === 0) {
                sendStop();
                return;
            }

            curS = Math.round(rawS * currentPower);
            curT = Math.round(rawT * currentPower);

            btn.classList.add('pressed');
            sendCommand(curS, curT);

            if (activeInterval) clearInterval(activeInterval);
            activeInterval = setInterval(() => {
                sendCommand(curS, curT);
            }, 120);
        }

        document.querySelectorAll('.ctrl-btn').forEach(btn => {
            btn.addEventListener('touchstart', (e) => { e.preventDefault(); startDrive(btn); });
            btn.addEventListener('touchend', (e) => { e.preventDefault(); sendStop(); });
            btn.addEventListener('mousedown', () => startDrive(btn));
            btn.addEventListener('mouseup', () => sendStop());
            btn.addEventListener('mouseleave', () => { if (btn.classList.contains('pressed')) sendStop(); });
        });

        window.addEventListener('keydown', (e) => {
            if (e.repeat) return;
            if (e.key === 'ArrowUp' || e.key === 'w') document.getElementById('btn-fwd').dispatchEvent(new Event('mousedown'));
            else if (e.key === 'ArrowDown' || e.key === 's') document.getElementById('btn-bwd').dispatchEvent(new Event('mousedown'));
            else if (e.key === 'ArrowLeft' || e.key === 'a') document.getElementById('btn-left').dispatchEvent(new Event('mousedown'));
            else if (e.key === 'ArrowRight' || e.key === 'd') document.getElementById('btn-right').dispatchEvent(new Event('mousedown'));
            else if (e.key === ' ' || e.key === 'Escape') sendStop();
        });

        window.addEventListener('keyup', (e) => {
            if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'w', 'a', 's', 'd'].includes(e.key)) {
                sendStop();
            }
        });
    </script>
</body>
</html>"#;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = socket.write_all(response.as_bytes()).await;
        }

        let _ = socket.flush().await;
        socket.close();
    }
}

// ════════════════════════════════════════════════════════════════
// Main Entry Point
// ════════════════════════════════════════════════════════════════

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Cấp phát heap cho Wi-Fi & Embassy Network Stack
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    // Embassy Timer Provider
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    // Status LED (GPIO2 - Onboard LED)
    let mut status_led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    // Motor 1 (Left): PWM GPIO 18, Dir1 GPIO 19, Dir2 GPIO 21
    let mut m1_pwm = Output::new(peripherals.GPIO18, Level::Low, OutputConfig::default());
    let mut m1_dir1 = Output::new(peripherals.GPIO19, Level::Low, OutputConfig::default());
    let mut m1_dir2 = Output::new(peripherals.GPIO21, Level::Low, OutputConfig::default());

    // Motor 2 (Right): PWM GPIO 5, Dir1 GPIO 17, Dir2 GPIO 16
    let mut m2_pwm = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let mut m2_dir1 = Output::new(peripherals.GPIO17, Level::Low, OutputConfig::default());
    let mut m2_dir2 = Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default());

    // Nút BOOT (GPIO0) với Pull-up
    let button = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Khởi tạo RCCarController
    let mut car_controller = RCCarController::with_config(RCCarConfig {
        max_speed: 100,
        steering_sensitivity: 1.0,
        smoothing_window: 5,
        signal_timeout_ms: 1000,
        loop_frequency_hz: 50,
    });
    car_controller.init();

    // Biến quản lý Wi-Fi SoftAP
    let mut wifi_peripheral = Some(peripherals.WIFI);
    let mut wifi_started = false;
    let mut wifi_active = false;

    // Biến nháy LED & Uptime
    let mut uptime_timer = Instant::now();
    let mut led_blink_timer = Instant::now();
    let mut led_blink_state = false;

    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║       ESP32 RC CAR — SMART WEB CONTROLLER        ║");
    println!("╠══════════════════════════════════════════════════╣");
    println!("║  Nhấn giữ nút BOOT 1.5s để BẬT / TẮT Wi-Fi SoftAP║");
    println!("║  LED ON khi Wi-Fi bật, NHÁY khi đang điều khiển ║");
    println!("║  LED TẮT khi đè BOOT tắt Wi-Fi                   ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    loop {
        // ─── 1. Cập nhật Uptime mỗi giây ───
        if uptime_timer.elapsed() >= Duration::from_secs(1) {
            CAR_UPTIME_SECS.fetch_add(1, Ordering::Relaxed);
            uptime_timer = Instant::now();
        }

        // ─── 2. Kiểm tra nút BOOT (GPIO0) bấm đè >= 1.5s để Bật/Tắt Wi-Fi ───
        if button.is_low() {
            let press_start = Instant::now();
            let mut triggered = false;

            while button.is_low() {
                if !triggered && press_start.elapsed() >= Duration::from_millis(1500) {
                    triggered = true;

                    if !wifi_started {
                        // Khởi động Wi-Fi SoftAP lần đầu
                        let ap_config = WifiConfig::AccessPoint(
                            AccessPointConfig::default()
                                .with_ssid("RC-Car".try_into().unwrap())
                                .with_authentication(
                                    esp_radio::wifi::AuthenticationMethodConfig::Wpa2Personal(
                                        "12345678".try_into().unwrap(),
                                    ),
                                ),
                        );
                        let controller_config =
                            ControllerConfig::default().with_initial_config(ap_config);
                        let wifi_peri = wifi_peripheral
                            .take()
                            .expect("Wi-Fi peripheral already used");
                        let mut wifi_controller = WifiController::new(wifi_peri, controller_config)
                            .expect("Wi-Fi init failed");
                        let _ = wifi_controller.set_max_tx_power(60);

                        let wifi_interface = Interface::access_point();
                        let net_config = Config::ipv4_static(StaticConfigV4 {
                            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 4, 1), 24),
                            gateway: Some(Ipv4Address::new(192, 168, 4, 1)),
                            dns_servers: Default::default(),
                        });

                        static STACK_RESOURCES: StaticCell<StackResources<8>> = StaticCell::new();
                        let resources = STACK_RESOURCES.init(StackResources::new());
                        let (stack, runner) =
                            embassy_net::new(wifi_interface, net_config, resources, 12345);

                        spawner.spawn(net_task(runner).unwrap());
                        spawner.spawn(dhcp_task(stack).unwrap());
                        spawner.spawn(dns_task(stack).unwrap());
                        spawner.spawn(web_server_task(stack).unwrap());

                        core::mem::forget(wifi_controller);

                        wifi_started = true;
                        wifi_active = true;
                        CAR_WIFI_ACTIVE.store(true, Ordering::Relaxed);

                        println!(
                            "⚡ [BOOT 1.5s] ĐÃ BẬT Wi-Fi SoftAP: SSID: ESP32-RC-Car | Pass: 12345678 | Web: http://192.168.4.1"
                        );
                    } else if wifi_active {
                        // Tắt sóng Wi-Fi SoftAP
                        unsafe {
                            esp_wifi_sys_esp32::include::esp_wifi_stop();
                        }
                        wifi_active = false;
                        CAR_WIFI_ACTIVE.store(false, Ordering::Relaxed);
                        CAR_ACTIVE_CONTROL.store(false, Ordering::Relaxed);
                        println!("🍃 [BOOT 1.5s] ĐÃ TẮT Wi-Fi SoftAP (Tắt sóng radio, LED tắt).");
                    } else {
                        // Bật lại sóng Wi-Fi
                        unsafe {
                            esp_wifi_sys_esp32::include::esp_wifi_start();
                        }
                        wifi_active = true;
                        CAR_WIFI_ACTIVE.store(true, Ordering::Relaxed);
                        println!("⚡ [BOOT 1.5s] ĐÃ BẬT LẠI Wi-Fi SoftAP.");
                    }
                }
                Timer::after(EmbassyDuration::from_millis(20)).await;
            }
        }

        // ─── 3. Logic điều khiển xe & Safety Timeout (50Hz - mỗi 20ms) ───
        let now_ms = CAR_UPTIME_SECS.load(Ordering::Relaxed).wrapping_mul(1000);
        let last_cmd_ms = CAR_LAST_CMD_MS.load(Ordering::Relaxed);
        let time_since_cmd = now_ms.saturating_sub(last_cmd_ms);

        let is_estop = CAR_EMERGENCY_STOP.load(Ordering::Relaxed);
        let is_wifi_on = CAR_WIFI_ACTIVE.load(Ordering::Relaxed);

        // Tự động dừng nếu mất lệnh quá 600ms (Watchdog Safety)
        if time_since_cmd > 600 && CAR_ACTIVE_CONTROL.load(Ordering::Relaxed) {
            CAR_ACTIVE_CONTROL.store(false, Ordering::Relaxed);
            CAR_STEER.store(0, Ordering::Relaxed);
            CAR_THROTTLE.store(0, Ordering::Relaxed);
        }

        let is_controlling = CAR_ACTIVE_CONTROL.load(Ordering::Relaxed) && !is_estop && is_wifi_on;

        let (left_speed, right_speed) = if is_controlling {
            let steer = CAR_STEER.load(Ordering::Relaxed) as i8;
            let throttle = CAR_THROTTLE.load(Ordering::Relaxed) as i8;
            car_controller.update(steer, throttle)
        } else {
            car_controller.update(0, 0);
            (0, 0)
        };

        CAR_SPEED_LEFT.store(left_speed as i32, Ordering::Relaxed);
        CAR_SPEED_RIGHT.store(right_speed as i32, Ordering::Relaxed);

        // Xuất tín hiệu ra các chân Motor Driver (H-Bridge)
        // Motor 1 (Trái):
        if left_speed > 0 {
            m1_dir1.set_high();
            m1_dir2.set_low();
            m1_pwm.set_high();
        } else if left_speed < 0 {
            m1_dir1.set_low();
            m1_dir2.set_high();
            m1_pwm.set_high();
        } else {
            m1_dir1.set_low();
            m1_dir2.set_low();
            m1_pwm.set_low();
        }

        // Motor 2 (Phải):
        if right_speed > 0 {
            m2_dir1.set_high();
            m2_dir2.set_low();
            m2_pwm.set_high();
        } else if right_speed < 0 {
            m2_dir1.set_low();
            m2_dir2.set_high();
            m2_pwm.set_high();
        } else {
            m2_dir1.set_low();
            m2_dir2.set_low();
            m2_pwm.set_low();
        }

        // ─── 4. Quản lý trạng thái đèn LED theo yêu cầu ───
        // - Wi-Fi tắt (hoặc đè boot tắt): Đèn TẮT hoàn toàn
        // - Wi-Fi bật & đang điều khiển: NHÁY LED liên tục (100ms)
        // - Wi-Fi bật & không điều khiển: Đèn SÁNG LIÊN TỤC
        if !is_wifi_on {
            status_led.set_low();
        } else if is_controlling {
            if led_blink_timer.elapsed() >= Duration::from_millis(100) {
                led_blink_state = !led_blink_state;
                status_led.set_level(if led_blink_state {
                    Level::High
                } else {
                    Level::Low
                });
                led_blink_timer = Instant::now();
            }
        } else {
            status_led.set_high();
        }

        Timer::after(EmbassyDuration::from_millis(20)).await;
    }
}
