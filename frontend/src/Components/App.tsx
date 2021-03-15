import React from "react";
import { BrowserRouter as Router, Switch, Route, NavLink } from "react-router-dom";
import "semantic-ui-css/semantic.min.css";
import { Menu, MenuItem, Image } from "semantic-ui-react";
import { AccountsComponent } from "./Accounts";
import { Import } from "./Import";
import { ImportAccounts } from "../Models/ImportAccount";
import "../App.css";
import { IncomeStatement } from "./Reports/IncomeStatement";

const App: React.FC = () => {
  return (
    <Router>
      <div className="flexbox">
        <Menu vertical className="sidebarmenu">
          <Menu.Item>
            <Menu.Header>Import</Menu.Header>
            <Menu.Menu>
              <MenuItem as={NavLink} to={"/accounts"} exact>
                Account overview
              </MenuItem>
              {ImportAccounts.map((x) => (
                <Menu.Item key={x.path} name={x.path} as={NavLink} to={`/import/${x.path}`}>
                  <Image src={x.icon} avatar />
                  {x.humanName}
                </Menu.Item>
              ))}
            </Menu.Menu>
          </Menu.Item>
          <Menu.Item>
            <Menu.Header>Reports</Menu.Header>
            <Menu.Menu>
              <Menu.Item as={NavLink} to={"/report/incomestatement"}>
                Income Statement
              </Menu.Item>
            </Menu.Menu>
          </Menu.Item>
        </Menu>
        <div className="main">
          <Switch>
            <Route path="/accounts" component={AccountsComponent} />
            <Route path="/import/:accountName" component={Import} />
            <Route path="/report/incomestatement" component={IncomeStatement} />
            <Route path="/" component={AccountsComponent} />
          </Switch>
        </div>
      </div>
    </Router>
  );
};

export default App;
