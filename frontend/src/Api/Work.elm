module Api.Work exposing (UpdateWork, Work, deleteWork, getWork, getWorkEvents, getWorks, postWork, putWork, workDecoder)

import Api exposing (ApiResource, Route(..), andThenDecode, apiResourceDecoder)
import Api.Clay exposing (Clay, clayDecoder)
import Api.Event exposing (Event, eventsDecoder)
import Api.State exposing (State, stateDecoder, stateEncoder)
import Http
import Json.Decode exposing (Decoder, field, int, list, map2, maybe, string, succeed, bool)
import Json.Decode.Extra exposing (datetime)
import Json.Encode as Encode
import Json.Encode.Extra as Encode
import Time exposing (Posix)


type alias CurrentState =
    { state : State
    , transitioned_at : Posix
    }


type alias Images =
    { header : Maybe String
    , thumbnail : Maybe String
    }


type alias Work =
    { id : Int
    , project : ApiResource
    , name : String
    , notes : Maybe String
    , clay : Clay
    , current_state : CurrentState
    , glaze_description : Maybe String
    , images : Images
    , created_at : Posix
    , is_multiple : Bool
    }

currentStateDecoder : Decoder CurrentState
currentStateDecoder =
    map2 CurrentState
        (field "state" stateDecoder)
        (field "transitioned_at" datetime)


imagesDecoder : Decoder Images
imagesDecoder =
    map2 Images
        (field "header" (maybe string))
        (field "thumbnail" (maybe string))

workDecoder : Decoder Work
workDecoder =
    succeed Work
        |> andThenDecode (field "id" int)
        |> andThenDecode (field "project" apiResourceDecoder)
        |> andThenDecode (field "name" string)
        |> andThenDecode (field "notes" (maybe string))
        |> andThenDecode (field "clay" clayDecoder)
        |> andThenDecode (field "current_state" currentStateDecoder)
        |> andThenDecode (field "glaze_description" (maybe string))
        |> andThenDecode (field "images" imagesDecoder)
        |> andThenDecode (field "created_at" datetime)
        |> andThenDecode (field "is_multiple" bool)


getWorks : { onResponse : Result Http.Error (List Work) -> msg } -> Cmd msg
getWorks options =
    Api.get
        { route = [ Works ]
        , expect = Http.expectJson options.onResponse (list workDecoder)
        }


getWork : Int -> { onResponse : Result Http.Error Work -> msg } -> Cmd msg
getWork id options =
    Api.get
        { route = [ Works, Id id ]
        , expect = Http.expectJson options.onResponse workDecoder
        }


getWorkEvents : Int -> { onResponse : Result Http.Error (List Event) -> msg } -> Cmd msg
getWorkEvents id options =
    Api.get
        { route = [ Works, Id id, Events ]
        , expect = Http.expectJson options.onResponse eventsDecoder
        }



-- PUT


type alias CreateWork =
    { project_id : Int
    , name : String
    , notes : Maybe String
    , clay_id : Int
    , glaze_description : Maybe String
    , state : State
    , thumbnail : Maybe String
    , header : Maybe String
    , is_multiple : Bool
    }


type alias UpdateWork =
    { project_id : Int
    , name : String
    , notes : Maybe String
    , clay_id : Int
    , glaze_description : Maybe String
    , thumbnail : Maybe String
    , header : Maybe String
    , is_multiple : Bool
    }


updateWorkEncoder : UpdateWork -> Encode.Value
updateWorkEncoder work =
    Encode.object
        [ ( "project_id", Encode.int work.project_id )
        , ( "name", Encode.string work.name )
        , ( "notes", Encode.maybe Encode.string work.notes )
        , ( "clay_id", Encode.int work.clay_id )
        , ( "glaze_description", Encode.maybe Encode.string work.glaze_description )
        , ( "thumbnail", Encode.maybe Encode.string work.thumbnail )
        , ( "header", Encode.maybe Encode.string work.header )
        , ( "is_multiple", Encode.bool work.is_multiple)
        ]


createWorkEncoder : CreateWork -> Encode.Value
createWorkEncoder work =
    Encode.object
        [ ( "project_id", Encode.int work.project_id )
        , ( "name", Encode.string work.name )
        , ( "notes", Encode.maybe Encode.string work.notes )
        , ( "clay_id", Encode.int work.clay_id )
        , ( "glaze_description", Encode.maybe Encode.string work.glaze_description )
        , ( "state", stateEncoder work.state )
        , ( "thumbnail", Encode.maybe Encode.string work.thumbnail )
        , ( "header", Encode.maybe Encode.string work.header )
        , ( "is_multiple", Encode.bool work.is_multiple)
        ]


putWork : Int -> UpdateWork -> { onResponse : Result Http.Error () -> msg } -> Cmd msg
putWork id work options =
    Api.put
        { route = [ Works, Id id ]
        , body = Http.jsonBody <| updateWorkEncoder work
        , expect = Http.expectWhatever options.onResponse
        }


postWork : CreateWork -> { onResponse : Result Http.Error Int -> msg } -> Cmd msg
postWork work options =
    Api.post
        { route = [ Works ]
        , body = Http.jsonBody <| createWorkEncoder work
        , expect = Http.expectJson options.onResponse int
        }


deleteWork : Int -> { onResponse : Result Http.Error () -> msg } -> Cmd msg
deleteWork id options =
    Api.delete
        { route = [ Works, Id id ]
        , expect = Http.expectWhatever options.onResponse
        }
