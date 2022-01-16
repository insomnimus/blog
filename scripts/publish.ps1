$articles = [ordered] @{
	"First Article" = "a1.md", "article", "wow"
	"Second Article" = "a2.md", "article"
}

foreach($entry in $articles.getEnumerator()) {
	$file, $tags = $entry.value
	if($tags) {
		$tags = @(
			"--tags"
			$tags | join-string -separator ","
		)
	}
	blog article publish $entry.name -f "$PSScriptRoot/$file" $tags
}
