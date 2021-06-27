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

export interface Price {
  amountFieldName?: string;
  currencyFieldName?: string;
}
export interface RulePosting {
  amountFieldName?: string;
  currencyFieldName?: string;
  price?: Price;
  account: string;
  negate: boolean;
  comment?: string;
}
