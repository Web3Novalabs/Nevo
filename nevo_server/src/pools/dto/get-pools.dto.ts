// TODO: Replace with real DTO validation/class-validator annotations from issue #17
export class GetPoolsDto {
  page?: string;
  limit?: string;
  search?: string;
  category?: string;
  status?: string;
}
