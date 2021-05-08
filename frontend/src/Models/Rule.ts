export interface Rule {
  id: number;
  priority: number;
  ruleName: string;
  descriptionTemplate: string;
  matchFieldName: string;
  matchFieldRegex: string;
  importAccount?: string;
  targetAccount: string;
}
