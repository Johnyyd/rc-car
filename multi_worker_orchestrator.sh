#!/bin/bash

MAX_RETRIES=3

# Đọc toàn bộ các task chưa hoàn thành vào một mảng để tránh mất biến trong subshell
mapfile -t TASKS < <(grep -E '^\s*-\s*\[ \]' TODO.md)

if [ ${#TASKS[@]} -eq 0 ]; then
    echo "Không tìm thấy task nào cần xử lý trong TODO.md."
    exit 0
fi

echo "Tìm thấy ${#TASKS[@]} tasks. Bắt đầu khởi tạo các Worker chạy song song..."

# Mảng lưu Process ID của các worker ngầm
PIDS=()
TASK_ID=1

for line in "${TASKS[@]}"; do
    # Bóc tách nội dung task
    TASK_CONTENT=$(echo "$line" | sed -E 's/^\s*-\s*\[ \]\s*//')
    
    # Định nghĩa ranh giới môi trường cho từng Worker
    BRANCH_NAME="worker-task-$TASK_ID"
    # Đặt thư mục worktree ngang hàng với thư mục gốc dự án để tránh đệ quy lồng nhau
    WORKTREE_DIR="../$BRANCH_NAME"
    LOG_FILE="../worker_${TASK_ID}.log"

    echo "▶ Đang cấp phát Git Worktree cho Worker $TASK_ID tại nhánh $BRANCH_NAME..."
    
    # Tạo nhánh mới từ nhánh hiện tại (bỏ qua lỗi nếu nhánh đã tồn tại)
    git branch $BRANCH_NAME 2>/dev/null || true
    # Khởi tạo không gian làm việc cô lập
    git worktree add $WORKTREE_DIR $BRANCH_NAME > /dev/null 2>&1

    # ==============================================================
    # KHỞI CHẠY WORKER NGẦM (Bọc trong () và dùng & để chạy song song)
    # ==============================================================
    (
        # Di chuyển worker vào thư mục làm việc riêng của nó
        cd $WORKTREE_DIR
        
        ATTEMPT=1
        SUCCESS=0
        CURRENT_PROMPT="$TASK_CONTENT"

        # Ghi log file riêng biệt, không in ra màn hình chính gây nhiễu
        echo "=== BẮT ĐẦU TASK $TASK_ID ===" > $LOG_FILE

        while [ $ATTEMPT -le $MAX_RETRIES ]; do
            echo "[Lần thử: $ATTEMPT/$MAX_RETRIES]" >> $LOG_FILE
            
            # Khởi chạy Claude Code. Mọi output (stdout/stderr) đẩy hết vào file log
            claude -p "$CURRENT_PROMPT" --dangerously-skip-permissions --yes >> $LOG_FILE 2>&1
            EXIT_CODE=$?
            
            if [ $EXIT_CODE -eq 0 ]; then
                SUCCESS=1
                # Tự động commit code thành công vào nhánh của worker này
                git add .
                git commit -m "Auto: Hoàn thành task $TASK_ID" >> $LOG_FILE 2>&1
                break
            else
                echo "Claude thất bại (Exit $EXIT_CODE). Thử lại..." >> $LOG_FILE
                # Loop Engineering: Cập nhật prompt
                CURRENT_PROMPT="[SYSTEM WARNING] Lần chạy trước thất bại. Hãy đọc kỹ file log và sửa lỗi. Nhiệm vụ gốc: $TASK_CONTENT"
                ATTEMPT=$((ATTEMPT + 1))
                sleep 3
            fi
        done

        if [ $SUCCESS -eq 1 ]; then
            echo "✅ Worker $TASK_ID đã hoàn thành! Code được lưu tại nhánh $BRANCH_NAME."
        else
            echo "❌ Worker $TASK_ID thất bại hoàn toàn sau $MAX_RETRIES lần thử."
        fi
    ) & # Ký tự '&' đưa toàn bộ khối lệnh trên vào chạy nền (Background)
    
    # Lưu lại PID của tiến trình nền vừa tạo
    PIDS+=($!)
    TASK_ID=$((TASK_ID + 1))
done

# ==============================================================
# HỆ THỐNG ĐIỀU PHỐI CHỜ ĐỢI
# ==============================================================
echo "⏳ Đang đợi tất cả các Worker xử lý xong. Bạn có thể mở các file ../worker_*.log để xem tiến độ..."

# Lệnh wait buộc bash script chính phải dừng lại cho đến khi tất cả các PIDS ngầm kết thúc
for pid in "${PIDS[@]}"; do
    wait $pid
done

echo "🎉 TOÀN BỘ WORKER ĐÃ DỪNG HOẠT ĐỘNG!"
echo "Bước tiếp theo: Hãy kiểm tra code ở các nhánh 'worker-task-*' và tiến hành git merge nếu code đạt yêu cầu."
