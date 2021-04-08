import { RecordedTransaction } from "./RecordedTransaction";
import { Rule } from "./Rule";

export interface RealTransaction {
  [key: string]: any;
}

export type RealTransactionField = keyof RealTransaction;
export type TransactionField = keyof RecordedTransaction;

export interface TransactionResponse {
  real_transaction: RealTransaction;
  recorded_transaction?: RecordedTransaction;
  rule?: Rule;
}

export interface ExistingTransactionResponse {
  real_transaction: RealTransaction | null;
  recorded_transaction: RecordedTransaction;
  real_cumulative: number;
  recorded_cumulative: number;
  errors: string[];
}
