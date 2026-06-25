import { Injectable, HttpException, HttpStatus } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';

@Injectable()
export class HorizonService {
  private readonly horizonUrl: string;

  constructor(private readonly configService: ConfigService) {
    this.horizonUrl = this.configService.get<string>(
      'HORIZON_URL',
      'https://horizon-testnet.stellar.org',
    ).replace(/\/$/, '');
  }

  /**
   * Fetches transactions for a specific contract/account ID from Horizon.
   * @param contractId The contract or account ID to fetch transactions for.
   * @param cursor The cursor to start fetching from (optional).
   * @returns Raw Horizon transaction records.
   */
  async getTransactions(contractId: string, cursor?: string): Promise<any[]> {
    try {
      const url = new URL(`${this.horizonUrl}/accounts/${contractId}/transactions`);
      url.searchParams.append('order', 'asc');
      if (cursor) {
        url.searchParams.append('cursor', cursor);
      }

      const response = await fetch(url.toString());
      if (!response.ok) {
        throw new HttpException(
          `Failed to fetch transactions from Horizon: ${response.statusText}`,
          response.status,
        );
      }

      const data = (await response.json()) as any;
      return data?._embedded?.records ?? [];
    } catch (error: any) {
      if (error instanceof HttpException) {
        throw error;
      }
      throw new HttpException(
        error.message || 'Internal server error while fetching transactions',
        HttpStatus.INTERNAL_SERVER_ERROR,
      );
    }
  }
}
