import React from "react";
import { Input, Menu } from "semantic-ui-react";

interface Props {
  defaultDate: Date;
  date: Date;
  onDateChanged: (date: Date) => void;
}

const DateRangeMenu: React.FC<Props> = ({ defaultDate, date, onDateChanged }) => {
  const tryParseDate = (date: string): Date => {
    try {
      const d = new Date(date);
      if (d instanceof Date && !isNaN(d.getTime())) {
        return d;
      }
    } catch {
      // fall back to return
    }
    return defaultDate;
  };

  return (
    <Menu attached="top">
      <Menu.Item>
        <Input
          label="From"
          type="date"
          value={toYMD(date)}
          onChange={(_, data) => onDateChanged(tryParseDate(data.value))}
        />
      </Menu.Item>
    </Menu>
  );
};

export default DateRangeMenu;

function toYMD(date: Date): string {
  const offset = date.getTimezoneOffset();
  date = new Date(date.getTime() - offset * 60 * 1000);
  return date.toISOString().split("T")[0];
}
