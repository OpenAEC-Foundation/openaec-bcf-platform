// Simple toast implementation for BCF Platform
// In a real implementation, this would integrate with a proper notification system

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export function showToast(message: string, type: ToastType = 'info'): void {
  // For now, just log to console and show alert for errors
  // This would be replaced with a proper toast UI component
  console.log(`[${type.toUpperCase()}] ${message}`);

  if (type === 'error') {
    // Show alert for errors so user sees them
    alert(message);
  }

  // TODO: Replace with proper toast UI component
  // Could use a library like react-hot-toast or implement custom component
}