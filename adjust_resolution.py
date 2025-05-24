#!/usr/bin/env python3
"""
分辨率自适应调节工具
这个脚本使用GTK来调整屏幕分辨率，适用于大多数Linux桌面环境
"""

import gi
import sys
import os
import subprocess

try:
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk, Gdk
except ImportError:
    print("错误: 未找到GTK库。请安装python3-gi包。")
    print("运行: sudo apt install python3-gi")
    sys.exit(1)

def get_screen_info():
    """获取当前屏幕信息"""
    display = Gdk.Display.get_default()
    screen = display.get_default_screen()
    monitor = display.get_primary_monitor()
    geometry = monitor.get_geometry()
    
    print(f"当前屏幕分辨率: {geometry.width}x{geometry.height}")
    print(f"屏幕数量: {display.get_n_monitors()}")
    
    return geometry.width, geometry.height

def set_resolution_with_xrandr(width, height):
    """使用xrandr设置分辨率"""
    try:
        # 获取当前连接的显示器
        output = subprocess.check_output(["xrandr"], universal_newlines=True)
        display_name = None
        
        for line in output.split('\n'):
            if " connected" in line:
                display_name = line.split()[0]
                break
        
        if not display_name:
            print("错误: 未找到连接的显示器")
            return False
            
        print(f"检测到显示器: {display_name}")
        
        # 生成modeline
        cvt_output = subprocess.check_output(["cvt", str(width), str(height)], universal_newlines=True)
        modeline = cvt_output.split('\n')[1].split('"')[1]
        mode_name = f"{width}x{height}_60.00"
        
        # 创建新模式
        subprocess.call(["xrandr", "--newmode", mode_name] + cvt_output.split('\n')[1].split(' ')[2:])
        
        # 添加模式到显示器
        subprocess.call(["xrandr", "--addmode", display_name, mode_name])
        
        # 切换到新模式
        subprocess.call(["xrandr", "--output", display_name, "--mode", mode_name])
        
        print(f"已设置分辨率为 {width}x{height}")
        return True
    except Exception as e:
        print(f"设置分辨率时出错: {e}")
        return False

def suggest_resolution(current_width, current_height):
    """根据当前分辨率建议更好的分辨率"""
    common_resolutions = [
        (1920, 1080),  # Full HD
        (1600, 900),   # HD+
        (1366, 768),   # WXGA
        (1280, 720),   # HD
        (1024, 768),   # XGA
        (800, 600),    # SVGA
    ]
    
    # 如果当前分辨率已经是最高的，就不需要调整
    if current_width >= 1920 and current_height >= 1080:
        return current_width, current_height
    
    # 找到比当前分辨率更高的最低分辨率
    for width, height in common_resolutions:
        if width > current_width and height > current_height:
            return width, height
    
    # 如果没有找到更高的分辨率，返回1280x720
    return 1280, 720

def main():
    print("分辨率自适应调节工具")
    print("====================")
    
    current_width, current_height = get_screen_info()
    
    if current_width <= 800 and current_height <= 600:
        print("检测到低分辨率，尝试调整...")
        target_width, target_height = suggest_resolution(current_width, current_height)
        
        print(f"尝试设置分辨率为: {target_width}x{target_height}")
        success = set_resolution_with_xrandr(target_width, target_height)
        
        if not success:
            print("无法自动设置分辨率。")
            print("建议手动安装显卡驱动以获得更好的分辨率支持。")
    else:
        print("当前分辨率看起来不错，无需调整。")

if __name__ == "__main__":
    main()
