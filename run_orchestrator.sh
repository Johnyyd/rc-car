#!/bin/bash

MAX_RETRIES=3

# Đọc từng dòng task chưa hoàn thành trong TODO.md
grep -E '^\s*-\s*\[ \]' TODO.md | while read -r line ; do
    # Bóc tách nội dung task (bỏ - [ ])
    TASK_CONTENT=$(echo "$line" | sed -E 's/^\s*-\s*\[ \]\s*//')
    echo "========================================"
    echo "▶ BẮT ĐẦU TASK: $TASK_CONTENT"
    
    ATTEMPT=1
    SUCCESS=0
    CURRENT_PROMPT="$TASK_CONTENT"

    while [ $ATTEMPT -le $MAX_RETRIES ]; do
        echo "[Lần thử: $ATTEMPT/$MAX_RETRIES]"
        
        # Chạy Claude Code ngầm định, truyền prompt vào
        claude -p "$CURRENT_PROMPT" --dangerously-skip-permissions
        
        # Bắt mã thoát
        EXIT_CODE=$?
        
        if [ $EXIT_CODE -eq 0 ]; then
            echo "✅ Task hoàn thành thành công!"
            SUCCESS=1
            # Đánh dấu task đã xong trong file TODO (đổi [ ] thành [x])
            sed -i "s/$line/- [x] $TASK_CONTENT/" TODO.md
            break
        else
            echo "❌ Claude thất bại (Exit code: $EXIT_CODE)."
            
            # LOOP ENGINEERING: Cập nhật prompt để AI nhận thức được sự thất bại
            CURRENT_PROMPT="[SYSTEM WARNING] Ở lần chạy trước, bạn đã thất bại và văng lỗi trước khi hoàn thành tiêu chí. Hãy xem lại toàn bộ file code vừa sửa, TÌM HIỂU LÝ DO THẤT BẠI và thử làm lại. Nhiệm vụ gốc: $TASK_CONTENT"
            
            ATTEMPT=$((ATTEMPT + 1))
            sleep 3 # Đợi 3 giây trước khi gọi API tiếp để tránh Rate Limit
        fi
    done

    if [ $SUCCESS -eq 0 ]; then
        echo "⛔ Đã thử $MAX_RETRIES lần nhưng vẫn thất bại. Dừng toàn bộ luồng để con người can thiệp!"
        exit 1
    fi
done

echo "🎉 TOÀN BỘ TODO ĐÃ HOÀN THÀNH!"
