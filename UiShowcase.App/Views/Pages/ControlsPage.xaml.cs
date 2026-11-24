using System.Windows;
using UiShowcase.ViewModels;

namespace UiShowcase.Views.Pages;

public partial class ControlsPage : System.Windows.Controls.UserControl
{
    public ControlsPage()
    {
        InitializeComponent();
    }

    private void SimulateInvalidInput_Click(object sender, RoutedEventArgs e)
    {
        if (DataContext is MainViewModel vm)
        {
            vm.DemoInputText = "invalid*path?";
            DemoPathInput.Focus();
        }
    }

    private void RestoreValidInput_Click(object sender, RoutedEventArgs e)
    {
        if (DataContext is MainViewModel vm)
        {
            vm.DemoInputText = @"D:\Codex\UiShowcase";
        }
    }
}
