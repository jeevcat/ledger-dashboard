import { RecordedTransaction } from "./RecordedTransaction";
import { Rule } from "./Rule";

export interface RealTransaction {
  [key: string]: any;
}

export type RealTransactionField = keyof RealTransaction;
export type TransactionField = keyof RecordedTransaction;

export interface ImportRow {
  real_transaction: RealTransaction | null;
  recorded_transaction?: RecordedTransaction;
  rule?: Rule;
  errors: string[];
}
