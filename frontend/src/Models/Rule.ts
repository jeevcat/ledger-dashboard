export interface Rule {
  id: number;
  priority: number;
  ruleName: string;
  matchFieldName: string;
  matchFieldRegex: string;
  account: string;
  descriptionTemplate: string;
}
