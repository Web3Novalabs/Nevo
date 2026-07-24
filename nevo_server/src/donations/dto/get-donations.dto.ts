import { DonationSortBy } from '../donations.service.js';

export class GetDonationsDto {
  page?: string;
  limit?: string;
  sortBy?: DonationSortBy;
}
