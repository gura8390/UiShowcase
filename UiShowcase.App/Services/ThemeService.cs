using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using UiShowcase.Models;
using Microsoft.Win32;

namespace UiShowcase.Services;

public sealed class ThemeService
{
    private const int DwmwaUseImmersiveDarkMode = 20;

    [DllImport("dwmapi.dll")]
    private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref int attrValue, int attrSize);

    public bool IsEffectiveDark { get; private set; }
    public bool IsHighContrast => SystemParameters.HighContrast;

    public void Apply(ThemeMode themeMode, DensityMode densityMode, AccentPreset accentPreset)
    {
        var app = System.Windows.Application.Current;
        if (app is null) return;

        var merged = app.Resources.MergedDictionaries;

        string colorUri;
        if (SystemParameters.HighContrast)
        {
            IsEffectiveDark = false;
            colorUri = "Themes/Colors.HighContrast.xaml";
        }
        else
        {
            IsEffectiveDark = themeMode switch
            {
                ThemeMode.Dark => true,
                ThemeMode.Light => false,
                _ => IsWindowsDarkTheme()
            };
            colorUri = IsEffectiveDark ? "Themes/Colors.Dark.xaml" : "Themes/Colors.Light.xaml";
        }

        var nextColorDict = new ResourceDictionary { Source = new Uri(colorUri, UriKind.Relative) };

        if (!SystemParameters.HighContrast)
        {
            ApplyAccentOverrides(nextColorDict, accentPreset, IsEffectiveDark);
        }

        var densityUri = densityMode == DensityMode.Compact
            ? "Themes/Density.Compact.xaml"
            : "Themes/Density.Standard.xaml";
        var nextDensityDict = new ResourceDictionary { Source = new Uri(densityUri, UriKind.Relative) };

        // 彻底移除旧的色彩与密度字典（包括 App.xaml 启动时初始加载的字典），避免后置字典遮蔽新主题
        for (var i = merged.Count - 1; i >= 0; i--)
        {
            var src = merged[i].Source?.OriginalString ?? string.Empty;
            if (src.Contains("Themes/Colors.", StringComparison.OrdinalIgnoreCase)
                || src.Contains("Themes/Density.", StringComparison.OrdinalIgnoreCase))
            {
                merged.RemoveAt(i);
            }
        }

        // 添加在列表末尾确保最高解析优先级
        merged.Add(nextColorDict);
        merged.Add(nextDensityDict);

        foreach (Window window in app.Windows)
        {
            ApplyWindowTitleBarTheme(window);
        }
    }

    public void ApplyWindowTitleBarTheme(Window window)
    {
        var hwnd = new WindowInteropHelper(window).Handle;
        if (hwnd == IntPtr.Zero) return;

        var darkValue = IsEffectiveDark && !SystemParameters.HighContrast ? 1 : 0;
        _ = DwmSetWindowAttribute(hwnd, DwmwaUseImmersiveDarkMode, ref darkValue, sizeof(int));
    }

    public static bool IsWindowsDarkTheme()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
            if (key?.GetValue("AppsUseLightTheme") is int value)
            {
                return value == 0;
            }
        }
        catch
        {
            // 回退浅色
        }

        return false;
    }

    private static void ApplyAccentOverrides(ResourceDictionary dict, AccentPreset preset, bool dark)
    {
        var (primary, hover, pressed, subtle, selected, onAccent) = (preset, dark) switch
        {
            (AccentPreset.Blue, false) => ("#0F6CBD", "#115EA3", "#0C4B85", "#EBF4FC", "#E6F1FB", "#FFFFFF"),
            (AccentPreset.Blue, true) => ("#4EA8F5", "#6BB7F7", "#3892E0", "#1D3247", "#1F364D", "#0D1117"),

            (AccentPreset.Teal, false) => ("#0E7A6D", "#0B6359", "#084E46", "#E7F6F3", "#E1F3F0", "#FFFFFF"),
            (AccentPreset.Teal, true) => ("#46C2B2", "#62CFC1", "#33A697", "#193532", "#1D3D39", "#0B1615"),

            (AccentPreset.Indigo, false) => ("#4F46E5", "#4338CA", "#3730A3", "#EEF0FE", "#E8EAFC", "#FFFFFF"),
            (AccentPreset.Indigo, true) => ("#818CF8", "#9CA5FA", "#6366F1", "#252647", "#2B2D54", "#0F1020"),

            (AccentPreset.Amber, false) => ("#B45309", "#92400E", "#78350F", "#FEF5E7", "#FDEFD9", "#FFFFFF"),
            (AccentPreset.Amber, true) => ("#F5B041", "#F7C065", "#D99529", "#3B2D16", "#423218", "#181207"),

            _ => ("#0F6CBD", "#115EA3", "#0C4B85", "#EBF4FC", "#E6F1FB", "#FFFFFF")
        };

        SetBrush(dict, "AccentPrimaryBrush", primary);
        SetBrush(dict, "AccentHoverBrush", hover);
        SetBrush(dict, "AccentPressedBrush", pressed);
        SetBrush(dict, "AccentSubtleBrush", subtle);
        SetBrush(dict, "BgSelectedBrush", selected);
        SetBrush(dict, "FocusRingBrush", dark ? hover : primary);
        SetBrush(dict, "TextOnAccentBrush", onAccent);
    }

    private static void SetBrush(ResourceDictionary dict, string key, string hex)
    {
        var color = (System.Windows.Media.Color)System.Windows.Media.ColorConverter.ConvertFromString(hex);
        dict[key] = new SolidColorBrush(color);
    }
}
