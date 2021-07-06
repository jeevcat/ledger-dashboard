import { asCurrency, asDate } from "../Utils/TextUtils";

export interface Amount {
  acommodity: string;
  aquantity: {
    floatingPoint: number;
    decimalPlaces: number;
    decimalMantissa: number;
  };
  aprice?: {
    tag: string;
    contents: Amount;
  };
}

export interface Posting {
  paccount: string;
  pamount: Amount[];
  ptags: string[][];
  pcomment: string;
}

export interface FormattedAmount {
  formatted: string;
  value: number;
  positive: boolean;
}

export const getPostingAmount = (p: Posting, negative?: boolean): FormattedAmount => {
  const amt = p.pamount[0];
  if (amt.aprice) {
    const value = amt.aprice.contents.aquantity.floatingPoint;
    const precision = amt.aprice.contents.aquantity.decimalPlaces;
    const cost = asCurrency(negative ? -value : value, amt.aprice.contents.acommodity, precision > 0, precision);

    const sym = amt.acommodity;
    const sym_amount = amt.aquantity.floatingPoint;
    const separator = amt.aprice.tag === "TotalPrice" ? "@@" : "@";

    return {
      formatted: `${sym_amount} ${sym} ${separator} ${cost}`,
      value,
      positive: value > 0,
    };
  }
  const value = amt.aquantity.floatingPoint;
  const precision = amt.aquantity.decimalPlaces;
  const cost = asCurrency(negative ? -value : value, amt.acommodity, precision > 0, precision);
  return {
    formatted: cost,
    value,
    positive: value > 0,
  };
};

export interface HledgerTransaction {
  tdescription: string;
  tdate: string;
  ttags?: string[][];
  tpostings?: Posting[];
}

export const getHledgerAmount = (t: HledgerTransaction, importAccountId: string): FormattedAmount | undefined => {
  const account = getMatchingAccount(t, importAccountId);
  if (!account) {
    return undefined;
  }
  for (const p of t.tpostings ?? []) {
    if (p.paccount.toLowerCase().includes(account.toLowerCase())) {
      return getPostingAmount(p);
    }
  }
};

export const getId = (t: HledgerTransaction): string | undefined => {
  if (t.ttags) {
    for (const tag of t.ttags) {
      if (tag[0] === "uuid") {
        return tag[1];
      }
    }
  }
  for (const p of t.tpostings ?? []) {
    for (const tag of p.ptags) {
      if (tag[0] === "uuid") {
        return tag[1];
      }
    }
  }
};

export const getDate = (t: HledgerTransaction): string => asDate(t.tdate);

export const getAccounts = (t: HledgerTransaction): string[] => (t.tpostings ?? []).map((p: Posting) => p.paccount);

export const getMatchingAccount = (t: HledgerTransaction, importAccountId: string): string | undefined => {
  return t.tpostings?.find((p: Posting) => p.paccount.toLowerCase().includes(importAccountId.toLowerCase()))?.paccount;
};

export const getUnmatchingAccounts = (t: HledgerTransaction, importAccountId: string): string[] =>
  (t.tpostings ?? [])
    .filter((p: Posting) => !p.paccount.toLowerCase().includes(":" + importAccountId.toLowerCase()))
    .map((p: Posting) => p.paccount);
