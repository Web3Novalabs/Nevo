import { create } from 'zustand';

type ModalType = 'donation' | 'createPool' | 'confirmTx' | null;

interface UIState {
  activeModal: ModalType;
  modalData: Record<string, unknown>;
  sidebarOpen: boolean;
  openModal: (type: ModalType, data?: Record<string, unknown>) => void;
  closeModal: () => void;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
}

export const useUIStore = create<UIState>()((set) => ({
  activeModal: null,
  modalData: {},
  sidebarOpen: false,

  openModal: (type, data = {}) => set({ activeModal: type, modalData: data }),
  closeModal: () => set({ activeModal: null, modalData: {} }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
}));
