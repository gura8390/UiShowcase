using System.Globalization;
using System.Windows;
using System.Windows.Data;
using System.Windows.Media;

namespace UiShowcase;

public sealed class BoolToVisibilityConverter : IValueConverter
{
    public bool Invert { get; set; }

    public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        var flag = value is bool b && b;
        if (Invert) flag = !flag;
        return flag ? Visibility.Visible : Visibility.Collapsed;
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

public sealed class ResourceBrushConverter : IValueConverter
{
    public object? Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        if (value is string key && System.Windows.Application.Current?.TryFindResource(key) is System.Windows.Media.Brush brush)
        {
            return brush;
        }

        return System.Windows.Media.Brushes.Transparent;
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

public sealed class StatusKindToBrushConverter : IValueConverter
{
    public string Role { get; set; } = "Foreground"; // Foreground, Background, Border

    public object? Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        var kind = value as string ?? "Info";
        var suffix = Role switch
        {
            "Background" => "BgBrush",
            "Border" => "BorderBrush",
            _ => "Brush"
        };

        var key = kind switch
        {
            "Success" => $"StatusSuccess{suffix}",
            "Warning" => $"StatusWarning{suffix}",
            "Danger" => $"StatusDanger{suffix}",
            _ => $"StatusInfo{suffix}"
        };

        return System.Windows.Application.Current?.TryFindResource(key) as System.Windows.Media.Brush
               ?? System.Windows.Media.Brushes.Transparent;
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        => throw new NotSupportedException();
}
