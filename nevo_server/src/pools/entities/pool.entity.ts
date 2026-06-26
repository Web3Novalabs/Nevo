// TODO: Replace with real TypeORM Entity from issue #16
export class Pool {
  id: string;
  title: string;
  description: string;
  category: string;
  status: 'Active' | 'Completed';
  target: number;
  raised: number;
  imageColor: string;
  creator?: string;
  createdAt: Date;
}
