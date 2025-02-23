import React, {FC, PropsWithChildren, useEffect} from 'react'
import ReactDOM from 'react-dom/client'
import {router} from './App'
import './index.css'
import {Provider} from "react-redux";
import {store} from "./store/store";
import {I18nextProvider} from "react-i18next";
import i18n from "./language/i18n";
import "@fortawesome/fontawesome-free/css/all.min.css";
import "@fontsource/roboto"
import "@fontsource/anton"
import {RouterProvider} from "react-router-dom";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "./utils/Utilities";
import {ConfigModel} from "./models/SysInfo";
import {setConfigModel} from "./store/CommonSlice";
import {useAppDispatch, useAppSelector} from "./store/hooks";
import {AuthProvider} from "react-oidc-context";
import {Loading} from "./components/Loading";
import {OIDCRefresher} from "./components/OIDCRefresher";
import {SnackbarProvider} from "notistack";

const AuthWrapper:FC<PropsWithChildren> = ({children})=>{
    const dispatch = useAppDispatch()
    const configModel = useAppSelector(state=>state.common.configModel)

    useEffect(()=>{
        axios.get(apiURL+"/sys/config").then((v:AxiosResponse<ConfigModel>)=>{
            dispatch(setConfigModel(v.data))
        })
    },[])

    if(configModel===undefined){
        return <Loading/>
    }

    if(configModel.oidcConfigured && configModel.oidcConfig){
        return <AuthProvider client_id={configModel.oidcConfig.clientId} authority={configModel.oidcConfig.authority} scope={configModel.oidcConfig.scope}
                      redirect_uri={configModel.oidcConfig.redirectUri}>
            <OIDCRefresher>
                {children}
            </OIDCRefresher>
        </AuthProvider>
    }

    return <>{children}</>
}




ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
      <I18nextProvider i18n={i18n}>
          <Provider store={store}>
              <AuthWrapper>
                  <SnackbarProvider maxSnack={4} >
                    <RouterProvider router={router}/>
                  </SnackbarProvider>
              </AuthWrapper>
          </Provider>
      </I18nextProvider>
  </React.StrictMode>,
)
