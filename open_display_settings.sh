#!/bin/bash

# 检测桌面环境
if [ "$XDG_CURRENT_DESKTOP" = "GNOME" ]; then
    echo "检测到GNOME桌面环境，打开显示设置..."
    gnome-control-center display
elif [ "$XDG_CURRENT_DESKTOP" = "KDE" ]; then
    echo "检测到KDE桌面环境，打开显示设置..."
    kcmshell5 kcm_kscreen
elif [ "$XDG_CURRENT_DESKTOP" = "XFCE" ]; then
    echo "检测到XFCE桌面环境，打开显示设置..."
    xfce4-display-settings
else
    echo "未能检测到桌面环境，尝试打开通用显示设置..."
    # 尝试几种常见的显示设置工具
    if command -v gnome-control-center &> /dev/null; then
        gnome-control-center display
    elif command -v arandr &> /dev/null; then
        arandr
    elif command -v lxrandr &> /dev/null; then
        lxrandr
    else
        echo "未找到可用的显示设置工具。"
        echo "尝试安装arandr: sudo apt install arandr"
        
        # 询问是否安装arandr
        read -p "是否安装arandr显示设置工具? (y/n) " answer
        if [ "$answer" = "y" ]; then
            sudo apt install -y arandr
            arandr
        fi
    fi
fi
