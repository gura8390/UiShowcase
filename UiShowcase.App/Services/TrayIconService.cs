using System.Drawing;
using System.IO;
using System.Windows.Forms;

namespace UiShowcase.Services;

public sealed class TrayIconService : IDisposable
{
    private readonly NotifyIcon _notifyIcon;
    private bool _disposed;

    public string TooltipText => _notifyIcon.Text;
    public bool IsVisible => _notifyIcon.Visible;

    public TrayIconService(
        Action onOpenMainWindow,
        Action onOpenSettings,
        Action onToggleTheme,
        Action onExit)
    {
        var icon = LoadAppIcon();
        var menu = new ContextMenuStrip();

        var openItem = new ToolStripMenuItem("打开主窗口");
        openItem.Font = new Font(openItem.Font, FontStyle.Bold);
        openItem.Click += (_, _) => onOpenMainWindow();

        var settingsItem = new ToolStripMenuItem("打开设置");
        settingsItem.Click += (_, _) => onOpenSettings();

        var toggleThemeItem = new ToolStripMenuItem("切换浅色 / 深色主题");
        toggleThemeItem.Click += (_, _) => onToggleTheme();

        var exitItem = new ToolStripMenuItem("退出");
        exitItem.Click += (_, _) => onExit();

        menu.Items.Add(openItem);
        menu.Items.Add(settingsItem);
        menu.Items.Add(toggleThemeItem);
        menu.Items.Add(new ToolStripSeparator());
        menu.Items.Add(exitItem);

        _notifyIcon = new NotifyIcon
        {
            Icon = icon,
            Text = "澄明界面展示",
            ContextMenuStrip = menu,
            Visible = true
        };

        _notifyIcon.DoubleClick += (_, _) => onOpenMainWindow();
    }

    public void ShowBalloon(string title, string message)
    {
        if (_disposed) return;
        _notifyIcon.ShowBalloonTip(2000, title, message, ToolTipIcon.Info);
    }

    private static Icon LoadAppIcon()
    {
        var iconPath = Path.Combine(AppContext.BaseDirectory, "Assets", "app.ico");
        if (File.Exists(iconPath))
        {
            return new Icon(iconPath);
        }

        return SystemIcons.Application;
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _notifyIcon.Visible = false;
        _notifyIcon.ContextMenuStrip?.Dispose();
        _notifyIcon.Icon?.Dispose();
        _notifyIcon.Dispose();
    }
}
