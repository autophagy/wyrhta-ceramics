module Auth exposing (User, onPageLoad)

import Auth.Action
import Dict
import Route exposing (Route)
import Route.Path
import Shared


type alias User =
    {}


{-| Called before an auth-only page is loaded.
-}
onPageLoad : Shared.Model -> Route () -> Auth.Action.Action User
onPageLoad shared route =
    if shared.authenticated then
        Auth.Action.loadPageWithUser {}

    else
        Auth.Action.pushRoute
            { path = Route.Path.Login
            , query = Dict.empty
            , hash = Nothing
            }
