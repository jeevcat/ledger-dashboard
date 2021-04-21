import { AlignedData } from "uplot";
import { HledgerTransaction } from "./HledgerTransaction";

export interface IncomeStatementResponse {
  data: AlignedData;
  topRevenues: HledgerTransaction[][];
  topExpenses: HledgerTransaction[][];
}
