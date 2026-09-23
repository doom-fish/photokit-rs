#import <Foundation/Foundation.h>
#import <Photos/Photos.h>

NS_ASSUME_NONNULL_BEGIN

typedef NS_ENUM(NSInteger, PKRFetchEntity) {
    PKRFetchEntityAsset = 0,
    PKRFetchEntityAssetCollection = 1,
    PKRFetchEntityCollectionList = 2,
    PKRFetchEntityCollection = 3,
};

NSPredicate * _Nullable PKRPredicateWithFormat(
    NSString *format,
    NSError * _Nullable * _Nullable error
);

BOOL PKRValidateFetchOptions(
    PHFetchOptions *options,
    PKRFetchEntity entity,
    NSError * _Nullable * _Nullable error
);

NSInteger PKRFetchResultCount(
    id fetchResult,
    NSError * _Nullable * _Nullable error
);

NS_ASSUME_NONNULL_END
