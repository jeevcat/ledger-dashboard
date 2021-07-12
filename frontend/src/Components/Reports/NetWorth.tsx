import React, { useState } from "react";
import { Segment } from "semantic-ui-react";
import DateRangeMenu from "./DateRangeMenu";
import { NetWorthPlot } from "./NetWorthPlot";

interface Props {}

const oneYearAgo = new Date();
oneYearAgo.setFullYear(oneYearAgo.getFullYear() - 1);

const NetWorth: React.FC<Props> = () => {
  const [startDate, setStartDate] = useState(oneYearAgo);
  return (
    <React.Fragment>
      <DateRangeMenu defaultDate={oneYearAgo} date={startDate} onDateChanged={setStartDate} />
      <Segment attached="bottom">
        <NetWorthPlot startDate={startDate} />
      </Segment>
    </React.Fragment>
  );
};

export default NetWorth;
