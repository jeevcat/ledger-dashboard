export interface Rule {
  id: number;
  priority: number;
  ruleName: string;
  matchFieldName: string;
  matchFieldRegex: string;
  importAccount: string;
  targetAccount: string;
  descriptionTemplate: string;
}
