#!/bin/bash

# NVIDIA驱动安装脚本
# 适用于RTX 5060 Ti 16GB显卡

echo "===== NVIDIA RTX 5060 Ti 驱动安装脚本 ====="
echo "此脚本将帮助您安装NVIDIA驱动以支持RTX 5060 Ti显卡"
echo "并设置2K分辨率(2560x1440)"
echo ""

# 检查是否以root权限运行
if [ "$EUID" -ne 0 ]; then
  echo "请以root权限运行此脚本"
  echo "使用: sudo ./install_nvidia_driver.sh"
  exit 1
fi

# 添加NVIDIA PPA
echo "正在添加NVIDIA驱动PPA..."
add-apt-repository ppa:graphics-drivers/ppa -y

# 更新软件包列表
echo "正在更新软件包列表..."
apt-get update

# 安装必要的软件包
echo "正在安装必要的软件包..."
apt-get install -y build-essential dkms

# 禁用nouveau驱动
echo "正在禁用nouveau驱动..."
if ! grep -q "blacklist nouveau" /etc/modprobe.d/blacklist-nouveau.conf 2>/dev/null; then
  echo "blacklist nouveau" > /etc/modprobe.d/blacklist-nouveau.conf
  echo "options nouveau modeset=0" >> /etc/modprobe.d/blacklist-nouveau.conf
  update-initramfs -u
fi

# 下载NVIDIA驱动
echo "正在下载NVIDIA驱动..."
cd /tmp
if [ -f "NVIDIA-Linux-x86_64-575.51.02.run" ]; then
  echo "驱动已下载，跳过下载步骤"
else
  echo "正在下载NVIDIA驱动 575.51.02..."
  wget https://us.download.nvidia.com/XFree86/Linux-x86_64/575.51.02/NVIDIA-Linux-x86_64-575.51.02.run
fi

# 如果575.51.02下载失败，尝试下载570.153.02
if [ ! -f "NVIDIA-Linux-x86_64-575.51.02.run" ]; then
  echo "下载575.51.02失败，尝试下载570.153.02..."
  wget https://us.download.nvidia.com/XFree86/Linux-x86_64/570.153.02/NVIDIA-Linux-x86_64-570.153.02.run
fi

# 检查驱动是否下载成功
if [ -f "NVIDIA-Linux-x86_64-575.51.02.run" ]; then
  DRIVER_FILE="NVIDIA-Linux-x86_64-575.51.02.run"
elif [ -f "NVIDIA-Linux-x86_64-570.153.02.run" ]; then
  DRIVER_FILE="NVIDIA-Linux-x86_64-570.153.02.run"
else
  echo "驱动下载失败，请检查网络连接或手动下载驱动"
  echo "您可以从NVIDIA官网下载驱动: https://www.nvidia.com/Download/index.aspx"
  exit 1
fi

# 安装NVIDIA驱动
echo "正在安装NVIDIA驱动..."
chmod +x $DRIVER_FILE
./$DRIVER_FILE --silent --dkms --no-opengl-files

# 创建xorg.conf文件
echo "正在创建xorg.conf文件..."
nvidia-xconfig --no-logo

# 创建分辨率设置脚本
echo "正在创建分辨率设置脚本..."
cat > /usr/local/bin/set-2k-resolution.sh << 'EOF'
#!/bin/bash

# 设置2K分辨率(2560x1440)
xrandr --newmode "2560x1440_60.00"  312.25  2560 2752 3024 3488  1440 1443 1448 1493 -hsync +vsync
xrandr --addmode $(xrandr | grep " connected" | cut -d" " -f1) "2560x1440_60.00"
xrandr --output $(xrandr | grep " connected" | cut -d" " -f1) --mode "2560x1440_60.00"
EOF

chmod +x /usr/local/bin/set-2k-resolution.sh

# 创建自启动脚本
echo "正在创建自启动脚本..."
mkdir -p /etc/xdg/autostart
cat > /etc/xdg/autostart/set-2k-resolution.desktop << EOF
[Desktop Entry]
Type=Application
Exec=/usr/local/bin/set-2k-resolution.sh
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
Name=Set 2K Resolution
Comment=Set 2K Resolution (2560x1440) at startup
EOF

echo "安装完成！请重启系统以应用更改。"
echo "重启后，系统将自动设置为2K分辨率(2560x1440)。"
echo "如果分辨率未自动设置，请运行: /usr/local/bin/set-2k-resolution.sh"
