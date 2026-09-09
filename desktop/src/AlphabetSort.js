//the first sort, and the plainest fuji will have: javascript's own sort(), capitals before lowercase and page10 before page9, with no locale and no opinion. sort.md carries the seven planned beside it

export default function alphabetSort(files) {//these image files as an ordered array of paths; a sort returns the order rather than a comparator, so a shuffle can be one too
	return files
		.map(file => file.path)//paths, because a path is what a view asks the cache with; within one folder, ordering these is ordering names
		.sort()//no comparator on purpose: here the platform's own answer is the wanted one
}
