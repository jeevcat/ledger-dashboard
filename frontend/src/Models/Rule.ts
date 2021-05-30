export interface Rule {
  id: number;
  priority: number;
  importerId: string;
  ruleName: string;
  matchFieldName: string;
  matchFieldRegex: string;
  descriptionTemplate: string;
  postings: RulePosting[];
}

export interface RulePosting {
  amountFieldName?: string;
  currencyFieldName?: string;
  account: string;
  negate?: boolean;
}
