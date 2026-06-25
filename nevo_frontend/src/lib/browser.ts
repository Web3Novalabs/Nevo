export async function copyTextToClipboard(text: string): Promise<void> {
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    await navigator.clipboard.writeText(text);
    return;
  }

  // Fallback to execCommand
  if (typeof document !== 'undefined' && document.execCommand) {
    const textArea = document.createElement('textarea');
    textArea.value = text;
    // Avoid scrolling to bottom
    textArea.style.top = '0';
    textArea.style.left = '0';
    textArea.style.position = 'fixed';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    try {
      const successful = document.execCommand('copy');
      if (!successful) {
        throw new Error('Copy command failed');
      }
    } catch {
      throw new Error('Copy command failed');
    } finally {
      document.body.removeChild(textArea);
    }
    return;
  }

  throw new Error('Copy command failed');
}

export function supportsMatchMedia(): boolean {
  return (
    typeof window !== 'undefined' && typeof window.matchMedia === 'function'
  );
}
