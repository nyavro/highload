# \DialogApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**dialog_user_id_list_get**](DialogApi.md#dialog_user_id_list_get) | **GET** /dialog/{user_id}/list | 
[**dialog_user_id_send_post**](DialogApi.md#dialog_user_id_send_post) | **POST** /dialog/{user_id}/send | 



## dialog_user_id_list_get

> Vec<models::DialogMessage> dialog_user_id_list_get(user_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |

### Return type

[**Vec<models::DialogMessage>**](DialogMessage.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dialog_user_id_send_post

> dialog_user_id_send_post(user_id, dialog_user_id_send_post_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |
**dialog_user_id_send_post_request** | Option<[**DialogUserIdSendPostRequest**](DialogUserIdSendPostRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

