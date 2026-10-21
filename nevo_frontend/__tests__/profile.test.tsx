import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

const mockFetchMyProfile = jest.fn();
const mockFetchMyDonations = jest.fn();
const mockUpdateProfile = jest.fn();
const mockToast = jest.fn();

jest.mock('@/lib/api-client', () => ({
    fetchMyProfile: mockFetchMyProfile,
    fetchMyDonations: mockFetchMyDonations,
    updateProfile: mockUpdateProfile,
}));

jest.mock('@/components/Toast', () => ({ toast: mockToast }));

jest.mock('@/src/store/walletStore', () => ({
    useWalletStore: () => ({ publicKey: 'GTESTWALLETADDRESS1234567890' }),
}));

import ProfilePage from '@/app/profile/page';

const profile = {
    publicKey: 'GTESTWALLETADDRESS1234567890',
    displayName: 'Ada Lovelace',
    createdAt: '2024-01-01T00:00:00.000Z',
};

async function openProfileEditor() {
    const user = userEvent.setup();
    render(<ProfilePage />);
    await user.click(await screen.findByRole('button', { name: 'Edit Profile' }));
    return user;
}

describe('profile display-name update flow', () => {
    beforeEach(() => {
        jest.clearAllMocks();
        mockFetchMyProfile.mockResolvedValue(profile);
        mockFetchMyDonations.mockResolvedValue([]);
        mockUpdateProfile.mockResolvedValue({ ...profile, displayName: 'Grace Hopper' });
    });

    it('pre-populates the form with the current display name', async () => {
        await openProfileEditor();

        expect(screen.getByLabelText('Display Name')).toHaveValue('Ada Lovelace');
    });

    it('submits the new name and shows a success state', async () => {
        const user = await openProfileEditor();
        const nameInput = screen.getByLabelText('Display Name');
        await user.clear(nameInput);
        await user.type(nameInput, 'Grace Hopper');
        await user.click(screen.getByRole('button', { name: 'Save' }));

        await waitFor(() => {
            expect(mockUpdateProfile).toHaveBeenCalledWith('Grace Hopper');
            expect(mockToast).toHaveBeenCalledWith('Profile updated successfully');
        });
        expect(
            await screen.findByRole('heading', { name: 'Grace Hopper' })
        ).toBeInTheDocument();
    });

    it.each([
        ['validation', 'Display name is required'],
        ['server', 'Profile service unavailable'],
    ])('shows a %s error inline without closing the form', async (_kind, message) => {
        mockUpdateProfile.mockRejectedValue(new Error(message));
        const user = await openProfileEditor();

        await user.click(screen.getByRole('button', { name: 'Save' }));

        expect(await screen.findByRole('alert')).toHaveTextContent(message);
        expect(screen.getByLabelText('Display Name')).toBeInTheDocument();
        expect(mockToast).not.toHaveBeenCalled();
    });
});