#!/bin/bash

INTEGRATION_BRANCH="integration-$(date +%s)"
git checkout main
git checkout -b "$INTEGRATION_BRANCH"

SUCCESS_MERGES=()
FAILED_MERGES=()

echo "Bắt đầu tiến trình gộp các nhánh Worker..."

# Lặp qua số lượng task (thay đổi giới hạn 12 tùy thực tế của bạn)
for i in {1..12}; do
    BRANCH="worker-task-$i"
    
    # Kiểm tra xem nhánh có tồn tại không
    if git show-ref --verify --quiet refs/heads/$BRANCH; then
        echo "▶ Đang thử gộp $BRANCH..."
        
        # Thử merge, tự động tạo commit message (--no-edit)
        if git merge "$BRANCH" --no-edit > /dev/null 2>&1; then
            echo "  ✅ Gộp thành công."
            SUCCESS_MERGES+=("$BRANCH")
        else
            echo "  ❌ PHÁT HIỆN XUNG ĐỘT (Conflict). Đang hoàn tác gộp nhánh này..."
            git merge --abort
            FAILED_MERGES+=("$BRANCH")
        fi
    fi
done

echo "====================================================="
echo "BÁO CÁO GỘP MÃ:"
echo "- Số nhánh gộp thành công: ${#SUCCESS_MERGES[@]}"
echo "- Số nhánh bị từ chối do xung đột: ${#FAILED_MERGES[@]} (${FAILED_MERGES[*]})"
echo "====================================================="

if [ ${#SUCCESS_MERGES[@]} -gt 0 ]; then
    echo "Môi trường hiện tại đang ở nhánh: $INTEGRATION_BRANCH"
    echo "YÊU CẦU: Hãy chạy test dự án (ví dụ: dotnet test / npm test / pytest) ngay bây giờ!"
    echo "Nếu mọi thứ hoạt động tốt, hãy chạy: git checkout main && git merge $INTEGRATION_BRANCH"
else
    echo "Không có code nào được gộp. Đang xóa nhánh tạm..."
    git checkout main
    git branch -D "$INTEGRATION_BRANCH"
fi