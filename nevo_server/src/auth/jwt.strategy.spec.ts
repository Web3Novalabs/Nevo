import { JwtStrategy } from './jwt.strategy';

describe('JwtStrategy', () => {
  let strategy: JwtStrategy;

  beforeEach(() => {
    process.env.JWT_SECRET = 'test-secret';
    strategy = new JwtStrategy();
  });

  it('returns sub and publicKey when both are present', () => {
    expect(strategy.validate({ sub: 'GSUB', publicKey: 'GPUBKEY' })).toEqual({
      sub: 'GSUB',
      publicKey: 'GPUBKEY',
    });
  });

  it('falls back to sub when publicKey is missing', () => {
    expect(strategy.validate({ sub: 'GSUB' })).toEqual({
      sub: 'GSUB',
      publicKey: 'GSUB',
    });
  });
});
