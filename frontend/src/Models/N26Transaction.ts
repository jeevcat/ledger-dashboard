import { Transaction, getId } from "./Transaction";
import { asDate } from "../Utils/TextUtils";

export interface N26Transaction {
  id: string;
  userId: string;
  type: string;
  amount: number;
  currencyCode: string;
  originalAmount: number;
  originalCurrency: string;
  exchangeRate: number;
  merchantCity: string;
  visibleTS: string;
  mcc: number;
  mccGroup: number;
  merchantName: string;
  recurring: boolean;
  accountId: string;
  category: string;
  cardId: string;
  userCertified: string;
  pending: boolean;
  transactionNature: string;
  createdTS: string;
  smartLinkId: string;
  linkId: string;
  confirmed: string;
  partnerBic: string;
  partnerName: string;
  partnerIban: string;
  referenceText: string;
  smartContactId: string;
}

export const getDate = (t: N26Transaction): string => asDate(t.visibleTS);

export const equals = (n26transaction: N26Transaction, transaction: Transaction): boolean => {
  return n26transaction.id === getId(transaction);
};
