/** @type {import('jest').Config} */
module.exports = {
  testEnvironment: "node",
  transform: {
    "^.+\\.tsx?$": [require.resolve("./nevo_frontend/node_modules/ts-jest"), { tsconfig: "./tsconfig.json" }],
  },
  testMatch: ["**/tests/**/*.test.ts"],
  moduleDirectories: ["node_modules", "nevo_frontend/node_modules"],
  moduleNameMapper: {
    "^@jest/globals$": require.resolve("./nevo_frontend/node_modules/@jest/globals"),
  },
};
