import { ImportAccountType } from "./ImportAccount";

export interface Rule {
  id: number;
  priority: number;
  ruleName: string;
  matchFieldName: string;
  matchFieldRegex: string;
  importAccount: ImportAccountType;
  targetAccount: string;
  descriptionTemplate: string;
}
