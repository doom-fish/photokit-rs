#import "PhotoKitObjCBridge.h"

static NSError *PKRErrorFromException(NSException *exception) {
    NSString *message = exception.reason ?: exception.name;
    return [NSError errorWithDomain:@"PhotoKitObjCBridge"
                               code:1
                           userInfo:@{NSLocalizedDescriptionKey: message}];
}

NSPredicate * _Nullable PKRPredicateWithFormat(
    NSString *format,
    NSError * _Nullable * _Nullable error
) {
    @try {
        return [NSPredicate predicateWithFormat:format argumentArray:@[]];
    } @catch (NSException *exception) {
        if (error != NULL) {
            *error = PKRErrorFromException(exception);
        }
        return nil;
    }
}

BOOL PKRValidateFetchOptions(
    PHFetchOptions *options,
    PKRFetchEntity entity,
    NSError * _Nullable * _Nullable error
) {
    @try {
        switch (entity) {
            case PKRFetchEntityAsset:
                (void)[PHAsset fetchAssetsWithOptions:options];
                break;
            case PKRFetchEntityAssetCollection:
                (void)[PHAssetCollection fetchAssetCollectionsWithType:PHAssetCollectionTypeAlbum
                                                               subtype:PHAssetCollectionSubtypeAny
                                                               options:options];
                break;
            case PKRFetchEntityCollectionList:
                (void)[PHCollectionList fetchCollectionListsWithType:PHCollectionListTypeFolder
                                                             subtype:PHCollectionListSubtypeAny
                                                             options:options];
                break;
            case PKRFetchEntityCollection:
                (void)[PHCollection fetchTopLevelUserCollectionsWithOptions:options];
                break;
        }
        return YES;
    } @catch (NSException *exception) {
        if (error != NULL) {
            *error = PKRErrorFromException(exception);
        }
        return NO;
    }
}

NSInteger PKRFetchResultCount(
    id fetchResult,
    NSError * _Nullable * _Nullable error
) {
    @try {
        return (NSInteger)[(PHFetchResult *)fetchResult count];
    } @catch (NSException *exception) {
        if (error != NULL) {
            *error = PKRErrorFromException(exception);
        }
        return -1;
    }
}
