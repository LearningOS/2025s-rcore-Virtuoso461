#!/bin/bash

# 自动调整分辨率脚本

# 获取当前连接的显示器
DISPLAY_NAME=$(xrandr | grep " connected" | cut -d" " -f1)
echo "检测到显示器: $DISPLAY_NAME"

# 尝试添加常用分辨率
echo "尝试添加常用分辨率..."

# 1920x1080
echo "添加 1920x1080 分辨率..."
cvt 1920 1080 60
xrandr --newmode "1920x1080_60.00"  173.00  1920 2048 2248 2576  1080 1083 1088 1120 -hsync +vsync
xrandr --addmode $DISPLAY_NAME "1920x1080_60.00"
xrandr --output $DISPLAY_NAME --mode "1920x1080_60.00"

# 如果上面的命令失败，尝试 1600x900
if [ $? -ne 0 ]; then
    echo "尝试 1600x900 分辨率..."
    cvt 1600 900 60
    xrandr --newmode "1600x900_60.00"  118.25  1600 1696 1856 2112  900 903 908 934 -hsync +vsync
    xrandr --addmode $DISPLAY_NAME "1600x900_60.00"
    xrandr --output $DISPLAY_NAME --mode "1600x900_60.00"
fi

# 如果上面的命令失败，尝试 1366x768
if [ $? -ne 0 ]; then
    echo "尝试 1366x768 分辨率..."
    cvt 1366 768 60
    xrandr --newmode "1366x768_60.00"   85.25  1366 1440 1576 1784  768 771 781 798 -hsync +vsync
    xrandr --addmode $DISPLAY_NAME "1366x768_60.00"
    xrandr --output $DISPLAY_NAME --mode "1366x768_60.00"
fi

# 如果上面的命令失败，尝试 1280x720
if [ $? -ne 0 ]; then
    echo "尝试 1280x720 分辨率..."
    cvt 1280 720 60
    xrandr --newmode "1280x720_60.00"   74.50  1280 1344 1472 1664  720 723 728 748 -hsync +vsync
    xrandr --addmode $DISPLAY_NAME "1280x720_60.00"
    xrandr --output $DISPLAY_NAME --mode "1280x720_60.00"
fi

echo "分辨率调整完成。如果屏幕仍然不正确，请尝试安装正确的显卡驱动。"
