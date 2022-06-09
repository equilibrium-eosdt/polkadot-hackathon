import { LoggerOptions, pino } from "pino";

const loggerOptions: LoggerOptions = {
  level: "info",
  timestamp: false,
  base: undefined,
  browser: {
    write: {
      trace: console.log,
      debug: console.log,
      info: console.info,
      warn: console.warn,
      error: console.error,
      fatal: console.error,
    },
  },
};

export const logger = pino(loggerOptions);
