<?php
/**
 * General Configuration
 *
 * All of your system's general configuration settings go in here. You can see a
 * list of the available settings in vendor/craftcms/cms/src/config/GeneralConfig.php.
 *
 * @see \craft\config\GeneralConfig
 */

use craft\config\GeneralConfig;
use craft\helpers\App;

/* craft base settings */
$baseSettings = GeneralConfig::create()
	// Set the default week start day for date pickers (0 = Sunday, 1 = Monday, etc.)
	->defaultWeekStartDay(1)
	// Prevent generated URLs from including "index.php"
	->omitScriptNameInUrls();

$customSettings = $baseSettings
	// convert any high-ASCII to their low ASCII counterparts (i.e. ñ → n).
	// Note that this only affects the JavaScript auto-generated slugs and they still can be manually entered in the slug.
	->limitAutoSlugsToAscii(true)
	->convertFilenamesToAscii(true)

	// Max upload size
	->maxUploadFileSize((float) (App::env("MAX_UPLOAD_FILE_SIZE") ?? 16777216))

	// Search term options
	->defaultSearchTermOptions([
		"subLeft" => true,
		"subRight" => true,
	])

	// Generate transforms before page load
	->generateTransformsBeforePageLoad(true)

	// Prevent transforms for gif and svg
	->transformGifs(false)
	->transformSvgs(false)
	->headlessMode(true)

	// automatically enable staticStatuses if CACHING is active
	// note that it will now require a cron job to call `craft update-statuses`
	// to update the status of an entry
	->staticStatuses(App::env("CACHING") === true);

return $customSettings;
