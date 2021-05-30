import { RealTransaction } from "./ImportRow";
import { RulePosting } from "./Rule";

export interface TransactionRequest {
  descriptionTemplate: string;
  sourceTransaction: RealTransaction;
  postings: RulePosting[];
  shouldWrite?: boolean;
}
