import React, { useMemo, useState } from 'react';
import { Search, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { Vendor } from './modelService';

export type VendorPickerSelectionMode = 'single' | 'multiple';

export function VendorPickerModal({
  vendors,
  title,
  searchPlaceholder,
  selectionMode = 'single',
  selectedVendorCodes = [],
  onSelectionChange,
  onSelect,
  onClose,
}: {
  vendors: readonly Vendor[];
  title: string;
  searchPlaceholder: string;
  selectionMode?: VendorPickerSelectionMode;
  selectedVendorCodes?: string[];
  onSelectionChange?: (vendorCodes: string[]) => void;
  onSelect?: (vendor: Vendor) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const [search, setSearch] = useState('');
  const selectedVendorCodeSet = useMemo(() => new Set(selectedVendorCodes), [selectedVendorCodes]);
  const filteredVendors = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) {
      return vendors;
    }
    return vendors.filter((vendor) => [
      vendor.name,
      vendor.vendorCode,
      vendor.description,
    ].some((value) => value.toLowerCase().includes(query)));
  }, [search, vendors]);

  const toggleVendorSelection = (vendor: Vendor) => {
    if (selectionMode === 'multiple') {
      const next = selectedVendorCodeSet.has(vendor.vendorCode)
        ? selectedVendorCodes.filter((vendorCode) => vendorCode !== vendor.vendorCode)
        : [...selectedVendorCodes, vendor.vendorCode];
      onSelectionChange?.(next);
      return;
    }
    onSelect?.(vendor);
  };

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-slate-950/55 p-4 backdrop-blur-sm">
      <div className="flex max-h-[88vh] w-full max-w-2xl flex-col overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-2xl dark:border-white/10 dark:bg-[#171719]">
        <div className="flex items-center justify-between border-b border-slate-200 px-5 py-4 dark:border-white/10">
          <div>
            <h4 className="text-lg font-semibold text-slate-900 dark:text-white">{title}</h4>
            <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{t('admin.model.mapping.form.vendorPicker.subtitle')}</p>
          </div>
          <button type="button" onClick={onClose} className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200">
            <X className="h-5 w-5" />
          </button>
        </div>
        <div className="space-y-4 p-5">
          <div className="relative">
            <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder={searchPlaceholder}
              className="w-full rounded-xl border border-slate-200 bg-white py-2.5 pl-10 pr-4 text-sm text-slate-900 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 dark:border-white/10 dark:bg-[#121214] dark:text-white"
            />
          </div>
          <div className="max-h-[420px] space-y-2 overflow-y-auto">
            {filteredVendors.length === 0 ? (
              <div className="rounded-xl border border-dashed border-slate-300 px-4 py-10 text-center text-sm text-slate-500 dark:border-white/10 dark:text-slate-400">
                {t('admin.model.mapping.form.noVendors')}
              </div>
            ) : filteredVendors.map((vendor) => {
              const checked = selectedVendorCodeSet.has(vendor.vendorCode);
              return (
                <button
                  key={vendor.id}
                  type="button"
                  onClick={() => toggleVendorSelection(vendor)}
                  className={`flex w-full items-center gap-3 rounded-2xl border px-4 py-3 text-left transition ${checked ? 'border-indigo-300 bg-indigo-50/80 shadow-sm ring-1 ring-indigo-100 dark:border-indigo-500/40 dark:bg-indigo-500/10 dark:ring-indigo-500/10' : 'border-slate-200 bg-slate-50 hover:border-indigo-300 hover:bg-indigo-50 dark:border-white/10 dark:bg-white/5 dark:hover:border-indigo-500/40 dark:hover:bg-indigo-500/10'}`}
                >
                  <span data-admin-vendor-picker-choice-control className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-white ring-1 ring-slate-200 dark:bg-[#171719] dark:ring-white/10">
                    <input
                      type={selectionMode === 'multiple' ? 'checkbox' : 'radio'}
                      checked={checked}
                      readOnly
                      tabIndex={-1}
                      className="h-4 w-4 rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
                    />
                  </span>
                  <span data-admin-vendor-picker-vendor-info className="min-w-0 flex-1">
                    <span className="block truncate font-semibold text-slate-900 dark:text-white">{vendor.name}</span>
                    <span className="block truncate text-xs text-slate-500">{vendor.vendorCode}</span>
                  </span>
                  <span data-admin-vendor-picker-vendor-status className="shrink-0 rounded-full bg-white px-3 py-1 text-xs font-semibold text-slate-500 ring-1 ring-slate-200 dark:bg-[#171719] dark:text-slate-300 dark:ring-white/10">
                    {vendor.status}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
        {selectionMode === 'multiple' && (
          <div className="flex items-center justify-between gap-3 border-t border-slate-200 px-5 py-4 dark:border-white/10">
            <div className="text-sm font-medium text-slate-600 dark:text-slate-300">
              {t('admin.model.site.form.vendorPickerSelectedCount', { count: selectedVendorCodes.length })}
            </div>
            <button
              type="button"
              onClick={onClose}
              className="rounded-xl bg-indigo-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-indigo-700"
            >
              {t('admin.model.site.form.vendorPickerDone')}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
