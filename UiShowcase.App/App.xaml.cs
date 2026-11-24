using System.IO;
using System.Threading;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using UiShowcase.Models;
using UiShowcase.Services;
using UiShowcase.ViewModels;
using UiShowcase.Views;
using Microsoft.Win32;

namespace UiShowcase;

public partial class App : System.Windows.Application
{
    private Mutex? _singleInstanceMutex;
    private EventWaitHandle? _activateEvent;
    private CancellationTokenSource? _activateListenerCts;

    private SettingsService? _settingsService;
    private AutoStartService? _autoStartService;
    private ThemeService? _themeService;
    private TrayIconService? _trayIconService;
    private MainViewModel? _mainViewModel;
    private MainWindow? _mainWindow;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        var isSnapshotMode = e.Args.Any(arg =>
            string.Equals(arg, "--render-snapshots", StringComparison.OrdinalIgnoreCase));

        if (!isSnapshotMode && !TryAcquireSingleInstance())
        {
            Shutdown(0);
            return;
        }

        _settingsService = new SettingsService();
        _autoStartService = new AutoStartService();
        _themeService = new ThemeService();

        _mainViewModel = new MainViewModel(
            _settingsService,
            _autoStartService,
            _themeService,
            ShowConfirmDialog);

        _mainWindow = new MainWindow(_mainViewModel, _themeService, ExitApplication);
        MainWindow = _mainWindow;

        if (!isSnapshotMode)
        {
            StartActivationListener();

            _trayIconService = new TrayIconService(
                onOpenMainWindow: () => Dispatcher.Invoke(() => _mainWindow.RestoreAndActivate()),
                onOpenSettings: () => Dispatcher.Invoke(() =>
                {
                    _mainViewModel.SelectedPageIndex = 3;
                    _mainWindow.RestoreAndActivate();
                }),
                onToggleTheme: () => Dispatcher.Invoke(() => _mainViewModel.QuickToggleLightDark()),
                onExit: () => Dispatcher.Invoke(ExitApplication));
        }

        SystemEvents.UserPreferenceChanged += SystemEvents_UserPreferenceChanged;

        var snapshotIdx = Array.FindIndex(e.Args, arg =>
            string.Equals(arg, "--render-snapshots", StringComparison.OrdinalIgnoreCase));
        if (snapshotIdx >= 0 && snapshotIdx + 1 < e.Args.Length)
        {
            var outDir = e.Args[snapshotIdx + 1];
            _mainWindow.Show();
            _ = RunVisualSnapshotVerificationAsync(outDir);
            return;
        }

        var startMinimized = e.Args.Any(arg =>
            string.Equals(arg, "--minimized", StringComparison.OrdinalIgnoreCase));

        if (!startMinimized)
        {
            _mainWindow.Show();
        }
    }

    private bool TryAcquireSingleInstance()
    {
        var baseName = Environment.GetEnvironmentVariable("JPMMVJUI_MUTEX_NAME");
        if (string.IsNullOrWhiteSpace(baseName))
        {
            baseName = "UiShowcase_SingleInstance";
        }

        var mutexName = $@"Local\{baseName}_Mutex";
        var eventName = $@"Local\{baseName}_ActivateEvent";

        _singleInstanceMutex = new Mutex(initiallyOwned: true, name: mutexName, out var createdNew);
        if (!createdNew)
        {
            try
            {
                using var existingEvent = EventWaitHandle.OpenExisting(eventName);
                existingEvent.Set();
            }
            catch
            {
                // 已有实例正在启动或退出中，直接结束重复进程
            }

            _singleInstanceMutex.Dispose();
            _singleInstanceMutex = null;
            return false;
        }

        _activateEvent = new EventWaitHandle(false, EventResetMode.AutoReset, eventName);
        return true;
    }

    private void StartActivationListener()
    {
        if (_activateEvent is null) return;

        _activateListenerCts = new CancellationTokenSource();
        var token = _activateListenerCts.Token;
        var waitHandles = new[] { _activateEvent, token.WaitHandle };

        var thread = new Thread(() =>
        {
            while (!token.IsCancellationRequested)
            {
                try
                {
                    var idx = WaitHandle.WaitAny(waitHandles);
                    if (idx == 0)
                    {
                        Dispatcher.BeginInvoke(() => _mainWindow?.RestoreAndActivate());
                    }
                    else
                    {
                        break;
                    }
                }
                catch
                {
                    break;
                }
            }
        })
        {
            IsBackground = true,
            Name = "SingleInstanceActivationListener"
        };

        thread.Start();
    }

    private async Task RunVisualSnapshotVerificationAsync(string outDir)
    {
        if (_mainWindow is null || _mainViewModel is null) return;
        Directory.CreateDirectory(outDir);

        async Task Capture(string fileName)
        {
            await Dispatcher.InvokeAsync(() => { }, DispatcherPriority.Render);
            await Task.Delay(200);
            _mainWindow.UpdateLayout();

            var root = _mainWindow.Content as FrameworkElement;
            if (root is null) return;

            var dpi = VisualTreeHelper.GetDpi(_mainWindow);
            var w = (int)Math.Ceiling(root.ActualWidth * dpi.DpiScaleX);
            var h = (int)Math.Ceiling(root.ActualHeight * dpi.DpiScaleY);
            if (w <= 0 || h <= 0) return;

            var rtb = new RenderTargetBitmap(w, h, dpi.PixelsPerInchX, dpi.PixelsPerInchY, PixelFormats.Pbgra32);
            rtb.Render(root);

            var encoder = new PngBitmapEncoder();
            encoder.Frames.Add(BitmapFrame.Create(rtb));
            using var fs = File.Open(Path.Combine(outDir, fileName), FileMode.Create);
            encoder.Save(fs);
        }

        _mainViewModel.SelectedPageIndex = 0;
        _mainViewModel.ThemeMode = ThemeMode.Light;
        _mainViewModel.DensityMode = DensityMode.Standard;
        _mainViewModel.AccentPreset = AccentPreset.Blue;
        await Capture("01_overview_light_standard.png");

        _mainViewModel.ThemeMode = ThemeMode.Dark;
        await Capture("02_overview_dark_standard.png");

        _mainViewModel.DensityMode = DensityMode.Compact;
        _mainViewModel.AccentPreset = AccentPreset.Teal;
        await Capture("03_overview_dark_compact_teal.png");

        _mainViewModel.ThemeMode = ThemeMode.Light;
        _mainViewModel.DensityMode = DensityMode.Standard;
        _mainViewModel.AccentPreset = AccentPreset.Blue;
        _mainViewModel.SelectedPageIndex = 1;
        await Capture("04_workbench_light.png");

        _mainViewModel.ThemeMode = ThemeMode.Dark;
        await Capture("05_workbench_dark.png");

        _mainViewModel.SelectedPageIndex = 2;
        await Capture("06_controls_dark.png");

        _mainViewModel.ThemeMode = ThemeMode.Light;
        _mainViewModel.SelectedPageIndex = 3;
        await Capture("07_settings_light.png");

        ExitApplication();
    }

    private bool ShowConfirmDialog(string title, string message, string confirmLabel)
    {
        if (_themeService is null) return false;

        var dialog = new ConfirmDialog(title, message, confirmLabel, _themeService)
        {
            Owner = _mainWindow
        };
        return dialog.ShowDialog() == true;
    }

    private void SystemEvents_UserPreferenceChanged(object sender, UserPreferenceChangedEventArgs e)
    {
        if (e.Category is UserPreferenceCategory.General or UserPreferenceCategory.Accessibility or UserPreferenceCategory.Color)
        {
            Dispatcher.Invoke(() => _mainViewModel?.RefreshSystemThemeIfNeeded());
        }
    }

    private void ReleaseSingleInstance()
    {
        try
        {
            _activateListenerCts?.Cancel();
            _activateListenerCts?.Dispose();
            _activateListenerCts = null;
        }
        catch { }

        try
        {
            _activateEvent?.Dispose();
            _activateEvent = null;
        }
        catch { }

        try
        {
            _singleInstanceMutex?.ReleaseMutex();
        }
        catch { }

        try
        {
            _singleInstanceMutex?.Dispose();
            _singleInstanceMutex = null;
        }
        catch { }
    }

    private void ExitApplication()
    {
        SystemEvents.UserPreferenceChanged -= SystemEvents_UserPreferenceChanged;
        _trayIconService?.Dispose();
        _trayIconService = null;

        if (_mainWindow is not null)
        {
            _mainWindow.AllowExitClose();
            _mainWindow.Close();
        }

        ReleaseSingleInstance();
        Shutdown();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        SystemEvents.UserPreferenceChanged -= SystemEvents_UserPreferenceChanged;
        _trayIconService?.Dispose();
        ReleaseSingleInstance();
        base.OnExit(e);
    }
}
