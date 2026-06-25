import { Injectable } from '@nestjs/common';
import { AuthGuard } from '@nestjs/passport';

@Injectable()
export class StellarAuthGuard extends AuthGuard('jwt') {}
