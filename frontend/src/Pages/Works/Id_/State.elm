module Pages.Works.Id_.State exposing (Model, Msg, page)

import Api
import Api.State exposing (State(..), enumState, putState, stateToString)
import Api.Work exposing (Work, getWork)
import Auth
import Effect exposing (Effect)
import Html exposing (Html)
import Html.Attributes as A
import Html.Events as E
import Http
import Page exposing (Page)
import Route exposing (Route)
import Shared
import View exposing (View)


page : Auth.User -> Shared.Model -> Route { id : String } -> Page Model Msg
page _ _ route =
    Page.new
        { init = init route.params.id
        , update = update
        , subscriptions = subscriptions
        , view = view
        }



-- INIT


type alias Model =
    { workData : Api.Data Work
    , id : Int
    , current_state : State
    , updateState : Maybe (Api.Data ())
    }


init : String -> () -> ( Model, Effect Msg )
init id_ _ =
    let
        id =
            Maybe.withDefault 0 <| String.toInt id_
    in
    ( { workData = Api.Loading
      , id = id
      , current_state = Unknown
      , updateState = Nothing
      }
    , Effect.sendCmd <| getWork id { onResponse = ApiRespondedWork }
    )



-- UPDATE


stringToState : String -> State
stringToState s =
    case s of
        "thrown" ->
            Thrown

        "being trimmed" ->
            Trimming

        "recycled" ->
            Recycled

        "awaiting bisque firing" ->
            AwaitingBisqueFiring

        "awaiting glaze firing" ->
            AwaitingGlazeFiring

        "finished" ->
            Finished

        _ ->
            Unknown


type Msg
    = ApiRespondedWork (Result Http.Error Work)
    | StateUpdated String
    | UpdateState
    | ApiResponededUpdateState (Result Http.Error ())


update : Msg -> Model -> ( Model, Effect Msg )
update msg model =
    case msg of
        ApiRespondedWork (Ok work) ->
            ( { model | workData = Api.Success work, current_state = work.current_state.state }
            , Effect.none
            )

        StateUpdated state ->
            ( { model | current_state = stringToState state }
            , Effect.none
            )

        UpdateState ->
            ( { model | updateState = Just Api.Loading }
            , Effect.sendCmd <| putState model.id model.current_state { onResponse = ApiResponededUpdateState }
            )

        ApiResponededUpdateState (Ok ()) ->
            ( { model | updateState = Just <| Api.Success () }, Effect.none )

        _ ->
            ( model, Effect.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- VIEW


stateToOption : State -> State -> Html Msg
stateToOption selected state =
    Html.option [ A.value <| stateToString state, A.selected (selected == state) ] [ Html.text <| stateToString state ]


viewStates : Model -> Html Msg
viewStates model =
    Html.select [ E.onInput StateUpdated ] (List.map (stateToOption model.current_state) enumState)


viewWorkDetails : Model -> Html Msg
viewWorkDetails model =
    let
        buttonText =
            case model.updateState of
                Nothing ->
                    "Update"

                Just (Api.Success _) ->
                    "Updated!"

                _ ->
                    "..."
    in
    Html.div [ A.class "container" ]
        [ Html.h1 [] [ Html.text <| "Editing Work State [" ++ String.fromInt model.id ++ "]" ]
        , Html.div [ A.class "settings work-settings" ]
            [ Html.div [ A.class "left" ]
                [ Html.h2 [] [ Html.text "State" ]
                , viewStates model
                ]
            ]
        , Html.button [ E.onClick UpdateState ] [ Html.text buttonText ]
        ]


view : Model -> View Msg
view model =
    { title = "Editing Project [" ++ String.fromInt model.id ++ "]"
    , body = [ viewWorkDetails model ]
    }
