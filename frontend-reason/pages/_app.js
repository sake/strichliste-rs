import '../styles/index.css';
import { fetchSettings, defaultSettings, make as Settings } from '../components/Settings.bs';
import { React } from 'react';
import App from 'next/app';


class MyApp extends App {
  state = {
    settings: defaultSettings,
    loaded: false,
  }

  componentDidMount = () => {
    if (!this.state.loaded) {
      fetchSettings(s => {
        this.setState({
          settings: s,
          loaded: true,
        });
      });
    }
  };

  render() {
    const { Component, pageProps } = this.props;
    // console.log(this.state.settings);

    return <Settings value={this.state.settings}>
      <Component {...pageProps} />
    </Settings>
  }
}

export default MyApp
