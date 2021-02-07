import React from "react";
import { BrowserRouter as Router, Switch, Route } from "react-router-dom";
import "semantic-ui-css/semantic.min.css";
import { AccountsComponent } from "./Accounts";
import { Import } from "./Import";

const App: React.FC = () => {
  return (
    <Router>
      <Switch>
        <Route path="/accounts" component={AccountsComponent} />
        <Route path="/import/:accountName" component={Import} />
        <Route path="/" component={AccountsComponent} />
      </Switch>
    </Router>
  );
};

export default App;
