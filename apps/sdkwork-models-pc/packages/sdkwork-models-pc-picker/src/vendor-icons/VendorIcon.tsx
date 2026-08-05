import { getVendorIconColor, resolveVendorIconKey } from './vendorIconCatalog';
import { getVendorIconSvg } from './vendorIconSvgs';
import './vendor-icons.css';

export interface VendorIconProps {
  /** Explicit cc-switch icon key; resolved from `vendorCode` when absent. */
  iconKey?: string;
  /** sdkwork-models vendor code (for example `deepseek` or `moonshot`). */
  vendorCode?: string;
  /** Display name used for the initials fallback chip. */
  name: string;
  /** Icon edge length in px or any CSS size string. Defaults to 16. */
  size?: number | string;
  /** Extra class applied to the root element. */
  className?: string;
  /** Render an initials chip when no icon resolves. Defaults to true. */
  showFallback?: boolean;
}

/**
 * Vendor brand icon for model pickers. Icons are copied from the cc-switch
 * library and rendered inline so monochrome (currentColor) icons pick up the
 * metadata default color or the surrounding text color. Unknown vendors render
 * a two-letter initials chip, matching cc-switch ProviderIcon behavior.
 */
export function VendorIcon({
  iconKey,
  vendorCode,
  name,
  size = 16,
  className = '',
  showFallback = true,
}: VendorIconProps) {
  const resolvedKey = iconKey ?? resolveVendorIconKey(vendorCode);
  const svg = resolvedKey ? getVendorIconSvg(resolvedKey) : '';
  const sizeStyle = {
    width: size,
    height: size,
    fontSize: size,
    lineHeight: 1,
  };
  const title = name;

  if (svg) {
    const color = resolvedKey ? getVendorIconColor(resolvedKey) : undefined;
    return (
      <span
        aria-hidden="true"
        className={`sdkwork-vendor-icon ${className}`.trim()}
        style={{ ...sizeStyle, ...(color ? { color } : null) }}
        title={title}
        dangerouslySetInnerHTML={{ __html: svg }}
      />
    );
  }

  if (showFallback) {
    const initials = name
      .split(/\s+/)
      .map((word) => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
    const fallbackFontSize = typeof size === 'number'
      ? `${Math.max(size * 0.5, 10)}px`
      : '0.5em';
    return (
      <span
        aria-hidden="true"
        className={`sdkwork-vendor-icon sdkwork-vendor-icon--fallback ${className}`.trim()}
        style={sizeStyle}
        title={title}
      >
        <span style={{ fontSize: fallbackFontSize }}>{initials}</span>
      </span>
    );
  }

  return null;
}
