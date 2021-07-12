import React, { useState } from "react";
import { Segment } from "semantic-ui-react";
import DateRangeMenu from "./DateRangeMenu";
import { IncomeStatementPlot } from "./IncomeStatementPlot";

interface Props {}

const oneYearAgo = new Date();
oneYearAgo.setFullYear(oneYearAgo.getFullYear() - 1);

const IncomeStatement: React.FC<Props> = () => {
  const [startDate, setStartDate] = useState(oneYearAgo);
  return (
    <React.Fragment>
      <DateRangeMenu defaultDate={oneYearAgo} date={startDate} onDateChanged={setStartDate} />
      <Segment attached="bottom">
        <IncomeStatementPlot startDate={startDate} />
      </Segment>
    </React.Fragment>
  );
};

export default IncomeStatement;
