import { getRequestId, requestContext } from './request-context';

describe('request-context', () => {
  describe('getRequestId()', () => {
    it('returns undefined when called outside of requestContext.run()', () => {
      expect(getRequestId()).toBeUndefined();
    });

    it('returns the stored requestId when called inside requestContext.run()', (done) => {
      const testRequestId = 'test-request-id-123';

      requestContext.run({ requestId: testRequestId }, () => {
        expect(getRequestId()).toBe(testRequestId);
        done();
      });
    });

    it('returns undefined again after requestContext.run() has exited', (done) => {
      const testRequestId = 'ephemeral-request-id';

      requestContext.run({ requestId: testRequestId }, () => {
        // inside the context — sanity check
        expect(getRequestId()).toBe(testRequestId);
        done();
      });

      // outside the context — must be undefined
      expect(getRequestId()).toBeUndefined();
    });

    it('isolates different requestIds across nested run() calls', (done) => {
      const outerRequestId = 'outer-id';
      const innerRequestId = 'inner-id';

      requestContext.run({ requestId: outerRequestId }, () => {
        expect(getRequestId()).toBe(outerRequestId);

        requestContext.run({ requestId: innerRequestId }, () => {
          expect(getRequestId()).toBe(innerRequestId);
          done();
        });
      });
    });

    describe('async callbacks within the same requestContext.run call', () => {
      it('correctly resolves requestId inside a setTimeout callback', (done) => {
        const testRequestId = 'async-test-id-1';

        requestContext.run({ requestId: testRequestId }, () => {
          setTimeout(() => {
            expect(getRequestId()).toBe(testRequestId);
            done();
          }, 10);
        });
      });

      it('correctly resolves requestId inside a Promise chain', (done) => {
        const testRequestId = 'promise-test-id-1';

        requestContext.run({ requestId: testRequestId }, () => {
          Promise.resolve()
            .then(() => {
              expect(getRequestId()).toBe(testRequestId);
            })
            .then(() => {
              done();
            });
        });
      });

      it('correctly resolves requestId inside nested Promise chains', (done) => {
        const testRequestId = 'nested-promise-id';

        requestContext.run({ requestId: testRequestId }, () => {
          Promise.resolve()
            .then(() => {
              expect(getRequestId()).toBe(testRequestId);
              return Promise.resolve();
            })
            .then(() => {
              expect(getRequestId()).toBe(testRequestId);
              return Promise.resolve();
            })
            .then(() => {
              expect(getRequestId()).toBe(testRequestId);
              done();
            });
        });
      });

      it('correctly resolves requestId inside async/await', (done) => {
        const testRequestId = 'async-await-id';

        const asyncFn = async () => {
          expect(getRequestId()).toBe(testRequestId);
          await Promise.resolve();
          expect(getRequestId()).toBe(testRequestId);
          done();
        };

        requestContext.run({ requestId: testRequestId }, () => {
          asyncFn();
        });
      });

      it('correctly resolves requestId across multiple setTimeout callbacks', (done) => {
        const testRequestId = 'multi-timeout-id';
        let callCount = 0;

        requestContext.run({ requestId: testRequestId }, () => {
          setTimeout(() => {
            expect(getRequestId()).toBe(testRequestId);
            callCount++;

            setTimeout(() => {
              expect(getRequestId()).toBe(testRequestId);
              callCount++;

              setTimeout(() => {
                expect(getRequestId()).toBe(testRequestId);
                expect(callCount).toBe(2);
                done();
              }, 5);
            }, 5);
          }, 5);
        });
      });
    });

    describe('concurrent requests isolation', () => {
      it('two concurrent requests never see each others requestId', (done) => {
        const requestId1 = 'request-id-1';
        const requestId2 = 'request-id-2';
        const results: { id: string; context: string | undefined }[] = [];

        const captureInContext = (id: string, delay: number) => {
          requestContext.run({ requestId: id }, () => {
            setTimeout(() => {
              results.push({
                id,
                context: getRequestId(),
              });

              if (results.length === 2) {
                // Verify each request only saw its own ID
                expect(
                  results.find((r) => r.id === requestId1)?.context,
                ).toBe(requestId1);
                expect(
                  results.find((r) => r.id === requestId2)?.context,
                ).toBe(requestId2);
                done();
              }
            }, delay);
          });
        };

        captureInContext(requestId1, 15);
        captureInContext(requestId2, 10);
      });

      it('two concurrent Promise chains never see each others requestId', (done) => {
        const requestId1 = 'promise-request-1';
        const requestId2 = 'promise-request-2';
        const results: { id: string; context: string | undefined }[] = [];

        const captureWithPromises = (id: string) => {
          requestContext.run({ requestId: id }, () => {
            Promise.resolve()
              .then(() => Promise.resolve())
              .then(() => {
                results.push({
                  id,
                  context: getRequestId(),
                });

                if (results.length === 2) {
                  expect(
                    results.find((r) => r.id === requestId1)?.context,
                  ).toBe(requestId1);
                  expect(
                    results.find((r) => r.id === requestId2)?.context,
                  ).toBe(requestId2);
                  done();
                }
              });
          });
        };

        captureWithPromises(requestId1);
        captureWithPromises(requestId2);
      });

      it('mixed async operations (setTimeout + Promise) never leak across requests', (done) => {
        const requestId1 = 'mixed-request-1';
        const requestId2 = 'mixed-request-2';
        const results: { id: string; contexts: (string | undefined)[] }[] = [];

        const captureWithMixed = (id: string) => {
          requestContext.run({ requestId: id }, () => {
            const contexts: (string | undefined)[] = [];

            // Immediate
            contexts.push(getRequestId());

            // Promise
            Promise.resolve().then(() => {
              contexts.push(getRequestId());
            });

            // setTimeout
            setTimeout(() => {
              contexts.push(getRequestId());
              results.push({ id, contexts });

              if (results.length === 2) {
                const res1 = results.find((r) => r.id === requestId1);
                const res2 = results.find((r) => r.id === requestId2);

                expect(res1?.contexts).toEqual([requestId1, requestId1, requestId1]);
                expect(res2?.contexts).toEqual([requestId2, requestId2, requestId2]);
                done();
              }
            }, 10);
          });
        };

        captureWithMixed(requestId1);
        captureWithMixed(requestId2);
      });

      it('three concurrent requests never cross-contaminate requestIds', (done) => {
        const requestIds = ['req-1', 'req-2', 'req-3'];
        const results: Record<string, string | undefined> = {};

        requestIds.forEach((id, index) => {
          requestContext.run({ requestId: id }, () => {
            setTimeout(() => {
              results[id] = getRequestId();

              if (Object.keys(results).length === 3) {
                expect(results['req-1']).toBe('req-1');
                expect(results['req-2']).toBe('req-2');
                expect(results['req-3']).toBe('req-3');
                done();
              }
            }, Math.random() * 20);
          });
        });
      });
    });
  });
});
