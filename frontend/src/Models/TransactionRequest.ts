import { RealTransaction } from "./ImportRow";

export interface TransactionRequest {
  account: string;
  descriptionTemplate: string;
  sourceTransaction: RealTransaction;
  shouldWrite?: boolean;
}
