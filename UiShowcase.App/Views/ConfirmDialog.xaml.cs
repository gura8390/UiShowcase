using System.Windows;
using System.Windows.Automation;
using UiShowcase.Services;

namespace UiShowcase.Views;

public partial class ConfirmDialog : Window
{
    public ConfirmDialog(string title, string message, string confirmLabel, ThemeService themeService)
    {
        InitializeComponent();
        Title = $"澄明界面展示 - {title}";
        TitleText.Text = title;
        MessageText.Text = message;
        ConfirmBtn.Content = confirmLabel;
        AutomationProperties.SetName(ConfirmBtn, confirmLabel);

        SourceInitialized += (_, _) => themeService.ApplyWindowTitleBarTheme(this);
        Loaded += (_, _) => CancelBtn.Focus();
    }

    private void ConfirmBtn_Click(object sender, RoutedEventArgs e)
    {
        DialogResult = true;
        Close();
    }

    private void CancelBtn_Click(object sender, RoutedEventArgs e)
    {
        DialogResult = false;
        Close();
    }
}
