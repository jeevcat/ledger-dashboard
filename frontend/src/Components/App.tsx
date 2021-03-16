import React from "react";
import { BrowserRouter as Router, NavLink, Route, Switch } from "react-router-dom";
import "semantic-ui-css/semantic.min.css";
import { Image, Menu } from "semantic-ui-react";
import "../App.css";
import { ImportAccounts } from "../Models/ImportAccount";
import { AccountsComponent } from "./Accounts";
import { Import } from "./Import";
import { IncomeStatement } from "./Reports/IncomeStatement";
import { NetWorth } from "./Reports/NetWorth";

const App: React.FC = () => {
  return (
    <Router>
      <div className="flexbox">
        <Menu vertical className="sidebarmenu">
          <Menu.Item as={NavLink} to={"/import"}>
            <Menu.Header>Import</Menu.Header>
            <Menu.Menu>
              {ImportAccounts.map((x) => (
                <Menu.Item key={x.path} as={NavLink} to={`/import/${x.path}`}>
                  <Image src={x.icon} inline rounded centered floated="right" style={{ width: "16px" }} />
                  {x.humanName}
                </Menu.Item>
              ))}
            </Menu.Menu>
          </Menu.Item>
          <Menu.Item as={NavLink} to={"/report"}>
            <Menu.Header>Reports</Menu.Header>
            <Menu.Menu>
              <Menu.Item as={NavLink} to={"/report/incomestatement"}>
                Income Statement
              </Menu.Item>
              <Menu.Item as={NavLink} to={"/report/networth"}>
                Net Worth
              </Menu.Item>
            </Menu.Menu>
          </Menu.Item>
        </Menu>
        <div className="main">
          <Switch>
            <Route path="/import/:accountName" component={Import} />
            <Route path="/import" component={AccountsComponent} />
            <Route path="/report/incomestatement" component={IncomeStatement} />
            <Route path="/report/networth" component={NetWorth} />
            <Route path="/" component={AccountsComponent} />
          </Switch>
        </div>
      </div>
    </Router>
  );
};

export default App;
