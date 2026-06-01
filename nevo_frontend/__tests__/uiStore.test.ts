import { useUIStore } from '@/src/store/uiStore';

beforeEach(() =>
  useUIStore.setState({ activeModal: null, modalData: {}, sidebarOpen: false })
);

describe('uiStore', () => {
  it('opens a modal with data', () => {
    useUIStore.getState().openModal('donation', { poolId: '1' });
    const { activeModal, modalData } = useUIStore.getState();
    expect(activeModal).toBe('donation');
    expect(modalData).toEqual({ poolId: '1' });
  });

  it('closes modal', () => {
    useUIStore.getState().openModal('createPool');
    useUIStore.getState().closeModal();
    expect(useUIStore.getState().activeModal).toBeNull();
  });

  it('toggles sidebar', () => {
    useUIStore.getState().toggleSidebar();
    expect(useUIStore.getState().sidebarOpen).toBe(true);
    useUIStore.getState().toggleSidebar();
    expect(useUIStore.getState().sidebarOpen).toBe(false);
  });

  it('setSidebarOpen sets value directly', () => {
    useUIStore.getState().setSidebarOpen(true);
    expect(useUIStore.getState().sidebarOpen).toBe(true);
  });
});
