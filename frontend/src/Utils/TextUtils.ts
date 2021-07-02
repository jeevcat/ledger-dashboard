export const toTitleCase = (text: string) =>
  text
    // insert a space between lower & upper
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    // space before last upper in a sequence followed by lower
    .replace(/\b([A-Z]+)([A-Z])([a-z])/, "$1 $2$3")
    // uppercase the first character
    .replace(/^./, function (str) {
      return str.toUpperCase();
    });

export function keys<O>(o: O) {
  return Object.keys(o) as (keyof O)[];
}

export function asEuro(amount: number, cents: boolean = true, precision: number = 2): string {
  return asCurrency(amount, "EUR", cents, precision);
}

export function asCurrency(amount: number, currency: string, cents: boolean = true, precision: number = 2): string {
  const formatter = new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: currency,
    maximumFractionDigits: cents ? precision : 0,
  });
  return formatter.format(amount);
}

export const asDate = (dateString: string) => {
  let date: Date = new Date(dateString);

  // Support for IB dates, e.g. 20210603;202600
  if (isNaN(date.getTime())) {
    // Ignore time
    const [d] = dateString.split(";");
    const yyyy = parseInt(d.substring(0, 4));
    const mm = parseInt(d.substring(4, 6));
    const dd = parseInt(d.substring(6, 8));
    date = new Date(yyyy, mm, dd);
  }

  return date.toLocaleDateString("en-DE");
};
