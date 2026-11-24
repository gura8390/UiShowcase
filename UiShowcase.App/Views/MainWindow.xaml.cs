using System.ComponentModel;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using UiShowcase.Services;
using UiShowcase.ViewModels;

namespace UiShowcase.Views;

public partial class MainWindow : Window
{
    private readonly ThemeService _themeService;
    private readonly Action _requestExit;
    private bool _allowClose;

    public MainWindow(MainViewModel viewModel, ThemeService themeService, Action requestExit)
    {
        InitializeComponent();
        DataContext = viewModel;
        _themeService = themeService;
        _requestExit = requestExit;

        ApplyAdaptiveInitialSize();

        SourceInitialized += (_, _) =>
        {
            _themeService.ApplyWindowTitleBarTheme(this);
            UpdateDisplayMetrics();
        };
        SizeChanged += (_, _) => UpdateDisplayMetrics();
        DpiChanged += (_, _) => UpdateDisplayMetrics();
        KeyDown += MainWindow_KeyDown;
    }

    public void AllowExitClose()
    {
        _allowClose = true;
    }

    public void RestoreAndActivate()
    {
        if (!IsVisible)
        {
            Show();
        }

        if (WindowState == WindowState.Minimized)
        {
            WindowState = WindowState.Normal;
        }

        Activate();
        Focus();
        _themeService.ApplyWindowTitleBarTheme(this);
    }

    protected override void OnClosing(CancelEventArgs e)
    {
        if (!_allowClose && DataContext is MainViewModel vm && vm.CloseToTray)
        {
            e.Cancel = true;
            Hide();
            vm.StatusMessage = "主窗口已隐藏到系统托盘，双击托盘图标或右键菜单可重新打开";
            return;
        }

        base.OnClosing(e);
    }

    private void ExitButton_Click(object sender, RoutedEventArgs e)
    {
        _requestExit();
    }

    private void MainWindow_KeyDown(object sender, System.Windows.Input.KeyEventArgs e)
    {
        if (e.Key == Key.Q && Keyboard.Modifiers == ModifierKeys.Control)
        {
            _requestExit();
            e.Handled = true;
        }
    }

    private void ApplyAdaptiveInitialSize()
    {
        var workArea = SystemParameters.WorkArea;
        var targetWidth = Math.Min(1260, Math.Max(820, workArea.Width * 0.82));
        var targetHeight = Math.Min(820, Math.Max(540, workArea.Height * 0.86));

        Width = Math.Min(targetWidth, workArea.Width * 0.92);
        Height = Math.Min(targetHeight, workArea.Height * 0.92);

        MinWidth = Math.Min(820, workArea.Width * 0.85);
        MinHeight = Math.Min(540, workArea.Height * 0.85);
    }

    private void UpdateDisplayMetrics()
    {
        if (DataContext is not MainViewModel vm) return;

        var dpi = VisualTreeHelper.GetDpi(this);
        var scalePercent = Math.Round(dpi.DpiScaleX * 100);
        var workArea = SystemParameters.WorkArea;

        vm.DisplayMetricsText =
            $"逻辑工作区：{workArea.Width:0} × {workArea.Height:0} DIP\n" +
            $"当前 DPI 缩放：{scalePercent:0}%（{dpi.PixelsPerInchX:0} DPI）\n" +
            $"当前窗口尺寸：{ActualWidth:0} × {ActualHeight:0} DIP（最小 {MinWidth:0} × {MinHeight:0}）";
    }
}
