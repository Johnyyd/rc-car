#!/bin/bash

MAX_RETRIES=3

mapfile -t TASKS < <(grep -E '^\s*-\s*\[ \]' TODO.md)

if [ ${#TASKS[@]} -eq 0 ]; then
    echo "Không tìm thấy task nào cần xử lý trong TODO.md."
    exit 0
fi

echo "Tìm thấy ${#TASKS[@]} tasks. Bắt đầu khởi tạo các Worker chạy song song..."

PIDS=()
TASK_ID=1

for line in "${TASKS[@]}"; do
    TASK_CONTENT=$(echo "$line" | sed -E 's/^\s*-\s*\[ \]\s*//')
    BRANCH_NAME="worker-task-$TASK_ID"
    WORKTREE_DIR="../$BRANCH_NAME"
    LOG_FILE="../worker_${TASK_ID}.log"

    echo "▶ Đang cấp phát Git Worktree cho Worker $TASK_ID tại nhánh $BRANCH_NAME..."
    
    # Dùng -B để force khởi tạo nhánh mới và in lỗi trực tiếp ra terminal nếu Git từ chối
    if ! git worktree add -B "$BRANCH_NAME" "$WORKTREE_DIR"; then
        echo "❌ LỖI CRITICAL: Git không thể tạo worktree cho Worker $TASK_ID. Hãy kiểm tra lại trạng thái 'git status'."
        exit 1
    fi

    (
        # Bắt buộc kiểm tra cd, nếu thư mục không tồn tại thì hủy ngay tiến trình con
        cd "$WORKTREE_DIR" || exit 1
        
        ATTEMPT=1
        SUCCESS=0
        CURRENT_PROMPT="$TASK_CONTENT"

        echo "=== BẮT ĐẦU TASK $TASK_ID ===" > "$LOG_FILE"

        while [ $ATTEMPT -le $MAX_RETRIES ]; do
            echo "[Lần thử: $ATTEMPT/$MAX_RETRIES]" >> "$LOG_FILE"
            
            claude -p "$CURRENT_PROMPT" --dangerously-skip-permissions >> "$LOG_FILE" 2>&1
            EXIT_CODE=$?
            
            if [ $EXIT_CODE -eq 0 ]; then
                SUCCESS=1
                git add .
                git commit -m "Auto: Hoàn thành task $TASK_ID" >> "$LOG_FILE" 2>&1
                break
            else
                echo "Claude thất bại (Exit $EXIT_CODE). Thử lại..." >> "$LOG_FILE"
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
    ) & 
    
    PIDS+=($!)
    TASK_ID=$((TASK_ID + 1))
done

echo "⏳ Đang đợi tất cả các Worker xử lý xong..."
for pid in "${PIDS[@]}"; do
    wait $pid
done

echo "🎉 TOÀN BỘ WORKER ĐÃ DỪNG HOẠT ĐỘNG!"
